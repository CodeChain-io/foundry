// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU Affero General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU Affero General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::common::*;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::types::*;
use coordinator::{Header, Transaction};
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use remote_trait_object::raw_exchange::{import_null_proxy, import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{service, Context as RtoContext, Service, ServiceRef};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

// Defines required types and traits

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    seq: TxSeq,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    NoSuchAccount,
    AccountExists,
    InvalidKey,
}

#[service]
pub trait AccountManager: Service {
    fn create_account(&mut self, public: &Public) -> Result<(), Error>;

    fn get_sequence(&self, public: &Public, default: bool) -> Result<TxSeq, Error>;

    fn increase_sequence(&mut self, public: &Public, default: bool) -> Result<(), Error>;
}

struct GetAccountAndSeq;

#[derive(Serialize, Deserialize, Debug)]
pub struct TxHello;
impl Action for TxHello {}
pub type OwnTransaction = crate::common::SignedTransaction<TxHello>;

// State machine and the module

struct Config {
    allow_hello: bool,
}

struct StateMachine {
    /// A configurations to control the way the machine works
    config: Arc<Config>,

    /// The state
    state: Box<dyn SubStorageAccess>,
    // It has no services imported from other modules.
}

impl Service for StateMachine {}

#[derive(Clone)]
struct GraphQlRoot {
    /// It holds a reference to the stateless machine
    config: Arc<Config>,

    // If there are state access services
    // imported from other modules in StateMachine,
    // GraphQlRoot has to carry those as well.s
    /// A service to obtain the state of arbitrary height
    get_storage: Arc<RwLock<dyn GetStorage>>,
}

impl Service for GraphQlRoot {}

struct GraphQlRequestHandler {
    root: GraphQlRoot,

    /// A runtime to process the asynchronous result of the query
    tokio_runtime: tokio::runtime::Runtime,
}

impl Service for GraphQlRequestHandler {}

pub struct Module {
    /// The executor of transactions.
    executor: Arc<RwLock<StateMachine>>,

    /// The handler of GraphQL requests
    query_handler: Arc<RwLock<GraphQlRequestHandler>>,
}

// state machine implementation

enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    InvalidSequence,
    AccountError(Error),
    NotAllowedHello,
}

impl StateMachine {
    fn get_account(&self, public: &Public) -> Result<Account, Error> {
        let bytes = match self.state.get(public.as_ref()) {
            Some(bytes) => bytes,
            None => return Err(Error::NoSuchAccount),
        };
        let account: Account = serde_cbor::from_slice(&bytes).map_err(|_| Error::InvalidKey)?;
        Ok(account)
    }

    fn excute_tx(&mut self, transaction: &Transaction) -> Result<(), ExecuteError> {
        if transaction.tx_type() != "account" {
            return Err(ExecuteError::InvalidMetadata)
        }

        let tx: OwnTransaction =
            serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;
        if self.get_sequence(&tx.signer_public, true).map_err(ExecuteError::AccountError)? != tx.tx.seq {
            return Err(ExecuteError::InvalidSequence)
        }
        if !self.config.allow_hello {
            return Err(ExecuteError::NotAllowedHello)
        }
        self.increase_sequence(&tx.signer_public, true).unwrap();
        Ok(())
    }
}

impl GraphQlRoot {
    fn create_state_machine(&self, height: Option<u64>) -> StateMachine {
        let state = self.get_storage.read().get_storage(height).unwrap().unwrap_import().into_proxy();
        StateMachine {
            config: Arc::clone(&self.config),
            state,
        }
    }
}

// service implementations

impl AccountManager for StateMachine {
    fn create_account(&mut self, public: &Public) -> Result<(), Error> {
        let account = Account {
            seq: 0,
        };
        if self.state.has(public.as_ref()) {
            return Err(Error::AccountExists)
        }
        self.state.set(public.as_ref(), serde_cbor::to_vec(&account).unwrap());
        Ok(())
    }

    fn get_sequence(&self, public: &Public, default: bool) -> Result<TxSeq, Error> {
        let bytes = match self.state.get(public.as_ref()) {
            Some(bytes) => bytes,
            None => {
                if default {
                    return Ok(0)
                } else {
                    return Err(Error::NoSuchAccount)
                }
            }
        };
        let account: Account = serde_cbor::from_slice(&bytes).map_err(|_| Error::InvalidKey)?;
        Ok(account.seq)
    }

    fn increase_sequence(&mut self, public: &Public, default: bool) -> Result<(), Error> {
        let option_bytes = self.state.get(public.as_ref());
        if option_bytes.is_none() {
            if default {
                self.create_account(public).expect("Synchronization bug in AccountManager");
                self.increase_sequence(public, false).expect("Synchronization bug in AccountManager");
                return Ok(())
            } else {
                return Err(Error::NoSuchAccount)
            }
        }
        let bytes = option_bytes.unwrap();
        let mut account: Account = serde_cbor::from_slice(&bytes).map_err(|_| Error::InvalidKey)?;
        account.seq += 1;
        self.state.set(public.as_ref(), serde_cbor::to_vec(&account).unwrap());
        Ok(())
    }
}

impl Service for GetAccountAndSeq {}
impl crate::sorting::GetAccountAndSeq for GetAccountAndSeq {
    fn get_account_and_seq(&self, tx: &Transaction) -> Result<(Public, TxSeq), ()> {
        assert_eq!(tx.tx_type(), "account");
        let tx: OwnTransaction = serde_cbor::from_slice(&tx.body()).map_err(|_| ())?;
        Ok((tx.signer_public, tx.tx.seq))
    }
}

impl TxOwner for StateMachine {
    fn block_opened(&mut self, _: &Header) -> Result<(), HeaderError> {
        Ok(())
    }

    fn execute_transaction(&mut self, transaction: &Transaction) -> Result<TransactionOutcome, ()> {
        if let Err(error) = self.excute_tx(transaction) {
            match error {
                ExecuteError::InvalidMetadata => Err(()),
                ExecuteError::InvalidSign => Err(()),
                ExecuteError::InvalidFormat => Err(()),
                ExecuteError::AccountError(_) => Err(()),
                ExecuteError::InvalidSequence => Err(()),
                ExecuteError::NotAllowedHello => Err(()),
            }
        } else {
            Ok(Default::default())
        }
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        assert_eq!(transaction.tx_type(), "account");
        let tx: OwnTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
        tx.verify().map_err(|_| todo_fixthis)?;
        Ok(())
    }

    fn block_closed(&mut self) -> Result<Vec<Event>, CloseBlockError> {
        Ok(Vec::new())
    }
}

impl Stateful for StateMachine {
    fn set_storage(&mut self, storage: ServiceRef<dyn SubStorageAccess>) {
        self.state = storage.unwrap_import().into_proxy();
    }
}

// Module implementation

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        let config = Arc::new(Config {
            // TODO: read this from argument
            allow_hello: true,
        });

        Module {
            executor: Arc::new(RwLock::new(StateMachine {
                config: Arc::clone(&config),
                state: import_null_proxy(),
            })),
            query_handler: Arc::new(RwLock::new(GraphQlRequestHandler {
                root: GraphQlRoot {
                    config,
                    get_storage: import_null_proxy(),
                },
                tokio_runtime: tokio::runtime::Runtime::new().unwrap(),
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "tx-owner" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.executor) as Arc<RwLock<dyn TxOwner>>)
            }
            "account-manager" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.executor) as Arc<RwLock<dyn AccountManager>>)
            }
            "stateful" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.executor) as Arc<RwLock<dyn Stateful>>)
            }
            "get-account-and-seq" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Box::new(GetAccountAndSeq) as Box<dyn crate::sorting::GetAccountAndSeq>)
            }
            "handle-graphql-request" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Arc::clone(&self.query_handler) as Arc<RwLock<dyn HandleGraphQlRequest>>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(&mut self, rto_context: &RtoContext, name: &str, handle: HandleToExchange) {
        match name {
            "get-storage" => {
                self.query_handler.write().root.get_storage = import_service_from_handle(rto_context, handle);
            }
            _ => panic!("Invalid name in import_service()"),
        }
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}

// Defines query handlers

#[async_graphql::Object]
impl GraphQlRoot {
    async fn with_block_height(&self, height: Option<u64>) -> StateMachine {
        self.create_state_machine(height)
    }
}

#[async_graphql::Object]
impl StateMachine {
    async fn account(&self, public: GqlPublic) -> Option<Account> {
        self.get_account(&public.0).ok()
    }
}

#[async_graphql::Object]
impl Account {
    async fn seq(&self) -> TxSeq {
        self.seq
    }
}

impl HandleGraphQlRequest for GraphQlRequestHandler {
    fn execute(&self, query: &str, variables: &str) -> String {
        handle_gql_query(self.tokio_runtime.handle(), self.root.clone(), query, variables)
    }
}
