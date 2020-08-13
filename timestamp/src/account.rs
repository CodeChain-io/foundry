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
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{service, Context as RtoContext, Service, ServiceRef};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Context {
    pub storage: Option<Box<dyn SubStorageAccess>>,
    pub allow_hello: bool,
}

impl Context {
    fn storage(&self) -> &dyn SubStorageAccess {
        self.storage.as_ref().unwrap().as_ref()
    }

    fn storage_mut(&mut self) -> &mut dyn SubStorageAccess {
        self.storage.as_mut().unwrap().as_mut()
    }
}

impl Service for Context {}

pub struct Module {
    ctx: Arc<RwLock<Context>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Error {
    NoSuchAccount,
    AccountExists,
    InvalidKey,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    seq: u64,
}

#[service]
pub trait AccountManager: Service {
    fn create_account(&mut self, public: &Public) -> Result<(), Error>;

    fn get_sequence(&self, public: &Public, default: bool) -> Result<u64, Error>;

    fn increase_sequence(&mut self, public: &Public, default: bool) -> Result<(), Error>;
}

impl AccountManager for Context {
    fn create_account(&mut self, public: &Public) -> Result<(), Error> {
        let account = Account {
            seq: 0,
        };
        if self.storage().has(public.as_ref()) {
            return Err(Error::AccountExists)
        }
        self.storage_mut().set(public.as_ref(), serde_cbor::to_vec(&account).unwrap());
        Ok(())
    }

    fn get_sequence(&self, public: &Public, default: bool) -> Result<u64, Error> {
        let bytes = match self.storage().get(public.as_ref()) {
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
        let option_bytes = self.storage().get(public.as_ref());
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
        self.storage_mut().set(public.as_ref(), serde_cbor::to_vec(&account).unwrap());
        Ok(())
    }
}

impl Stateful for Context {
    fn set_storage(&mut self, storage: ServiceRef<dyn SubStorageAccess>) {
        self.storage.replace(storage.unwrap_import().into_proxy());
    }
}

enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    InvalidSequence,
    AccountError(Error),
    NotAllowedHello,
}

impl Context {
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
        if !self.allow_hello {
            return Err(ExecuteError::NotAllowedHello)
        }
        self.increase_sequence(&tx.signer_public, true).unwrap();
        Ok(())
    }
}

impl TxOwner for Context {
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

#[derive(Serialize, Deserialize, Debug)]
pub struct TxHello;
impl Action for TxHello {}
pub type OwnTransaction = crate::common::SignedTransaction<TxHello>;

struct GetAccountAndSeq;
impl Service for GetAccountAndSeq {}
impl crate::sorting::GetAccountAndSeq for GetAccountAndSeq {
    fn get_account_and_seq(&self, tx: &Transaction) -> Result<(Public, u64), ()> {
        assert_eq!(tx.tx_type(), "account");
        let tx: OwnTransaction = serde_cbor::from_slice(&tx.body()).map_err(|_| ())?;
        Ok((tx.signer_public, tx.tx.seq))
    }
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            ctx: Arc::new(RwLock::new(Context {
                storage: None,
                // TODO: read this from config
                allow_hello: true,
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "tx_owner" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn TxOwner>>)
            }
            "account_manager" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn AccountManager>>)
            }
            "stateful" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn Stateful>>)
            }
            "get_account_and_seq" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Box::new(GetAccountAndSeq) as Box<dyn crate::sorting::GetAccountAndSeq>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(&mut self, rto_context: &RtoContext, name: &str, handle: HandleToExchange) {
        match name {
            "sub_storage_access" => {
                self.ctx.write().storage.replace(import_service_from_handle(rto_context, handle));
            }
            _ => panic!("Invalid name in import_service()"),
        }
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
