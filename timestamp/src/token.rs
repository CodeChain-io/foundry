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

use crate::account::AccountManager;
use crate::common::{self, SignedTransaction};
use ccrypto::blake256;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::types::*;
use coordinator::{Header, Transaction};
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use primitives::H256;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{service, Context as RtoContext, Service, ServiceRef};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use std::sync::Arc;

pub struct Context {
    pub account: Option<Box<dyn AccountManager>>,
    pub storage: Option<Box<dyn SubStorageAccess>>,
}

impl Context {
    fn account(&self) -> &dyn AccountManager {
        self.account.as_ref().unwrap().as_ref()
    }

    fn account_mut(&mut self) -> &mut dyn AccountManager {
        self.account.as_mut().unwrap().as_mut()
    }

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

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    /// The issuer is recorded in the Token.
    /// Since Token module is general, it can be used from various other modules.
    /// `issuer` is for preventing different tokens to get mixed in such case.
    ///
    /// Even in a same module, you could consider advanced scheme where you
    /// distribute tokens with various issuer for special purpose (e.g invalidatablity)
    pub issuer: H256,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Account {
    pub tokens: Vec<Token>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NoSuchAccount,
    InvalidKey,
}

#[service]
pub trait TokenManager: Service {
    fn get_account(&self, public: &Public) -> Result<Account, Error>;
    fn issue_token(&mut self, issuer: &H256, receiver: &Public) -> Result<(), Error>;
    fn get_owning_accounts_with_issuer(&self, issuer: &H256) -> Result<BTreeSet<Public>, Error>;
}

impl TokenManager for Context {
    fn get_account(&self, public: &Public) -> Result<Account, Error> {
        let x = self.storage().get(Self::get_key(public).as_bytes()).ok_or_else(|| Error::NoSuchAccount)?;
        Ok(serde_cbor::from_slice(&x).map_err(|_| Error::InvalidKey)?)
    }

    fn issue_token(&mut self, issuer: &H256, receiver: &Public) -> Result<(), Error> {
        let mut account = self.get_account_or_default(receiver).map_err(|_| Error::InvalidKey)?;
        account.tokens.push(Token {
            issuer: *issuer,
        });
        self.set_account(receiver, &account);
        let mut set = self.get_owning_accounts_with_issuer(issuer).map_err(|_| Error::InvalidKey)?;
        set.insert(*receiver);
        self.set_owning_accounts_with_issuer(issuer, set);

        Ok(())
    }

    fn get_owning_accounts_with_issuer(&self, issuer: &H256) -> Result<BTreeSet<Public>, Error> {
        Ok(if let Some(x) = self.storage().get(Self::get_key_account_set(issuer).as_bytes()) {
            serde_cbor::from_slice(&x).map_err(|_| Error::InvalidKey)?
        } else {
            BTreeSet::new()
        })
    }
}

#[derive(Debug)]
enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    AccountModuleError(crate::account::Error),
    InvalidSequence,
    NoAccount,
    InvalidKey,
    NoToken,
}

impl Context {
    fn get_key(key: &Public) -> H256 {
        blake256(&{
            let mut v = serde_cbor::to_vec(&key).unwrap();
            v.extend_from_slice(b"Token-Module-Account");
            v
        } as &[u8])
    }

    fn get_key_account_set(issuer: &H256) -> H256 {
        blake256(&{
            let mut v = serde_cbor::to_vec(&issuer).unwrap();
            v.extend_from_slice(b"Token-Module-Account-Set");
            v
        } as &[u8])
    }

    fn get_account_or_default(&self, key: &Public) -> Result<Account, ()> {
        if let Some(x) = self.storage().get(Self::get_key(key).as_bytes()) {
            Ok(serde_cbor::from_slice(&x).map_err(|_| ())?)
        } else {
            Ok(Account {
                tokens: Vec::new(),
            })
        }
    }

    /// set_account() must not fail
    fn set_account(&mut self, key: &Public, account: &Account) {
        self.storage_mut().set(Self::get_key(key).as_bytes(), serde_cbor::to_vec(account).unwrap());
    }

    /// set_owning_accounts_with_issuer() must not fail
    fn set_owning_accounts_with_issuer(&mut self, issuer: &H256, set: BTreeSet<Public>) {
        self.storage_mut().set(Self::get_key_account_set(issuer).as_bytes(), serde_cbor::to_vec(&set).unwrap());
    }

    fn excute_tx(&mut self, transaction: &Transaction) -> Result<(), ExecuteError> {
        if transaction.tx_type() != "token" {
            return Err(ExecuteError::InvalidMetadata)
        }
        let tx: OwnTransaction =
            serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;
        if self.account().get_sequence(&tx.signer_public, true).map_err(ExecuteError::AccountModuleError)? != tx.tx.seq
        {
            return Err(ExecuteError::InvalidSequence)
        }

        match tx.tx.action {
            Action::TransferToken(ActionTransferToken {
                receiver,
                issuer,
            }) => {
                let mut sender_account: Account = serde_cbor::from_slice(
                    &self
                        .storage()
                        .get(Self::get_key(&tx.signer_public).as_bytes())
                        .ok_or_else(|| ExecuteError::NoAccount)?,
                )
                .map_err(|_| ExecuteError::InvalidKey)?;

                let mut found = None;
                for (i, token) in sender_account.tokens.iter().enumerate() {
                    if token.issuer == issuer {
                        found = Some(i)
                    }
                }
                let index = found.ok_or_else(|| ExecuteError::NoToken)?;
                let token = sender_account.tokens.remove(index);
                let mut recipient_account =
                    self.get_account_or_default(&receiver).map_err(|_| ExecuteError::InvalidKey)?;

                let mut set = self.get_owning_accounts_with_issuer(&issuer).map_err(|_| ExecuteError::InvalidKey)?;
                // From now on, it will actually mutate the state and must not fail
                // to keep the consistency of the state.
                // This issue might be handled by the coordinator and should be decided in detail.

                // If that was the last token with the issuer
                if sender_account.tokens.iter().find(|&x| x.issuer == issuer).is_none() {
                    assert!(set.remove(&tx.signer_public));
                }
                set.insert(receiver);
                self.set_owning_accounts_with_issuer(&issuer, set);

                recipient_account.tokens.push(token);
                self.set_account(&tx.signer_public, &sender_account);
                self.set_account(&receiver, &recipient_account);
                self.account_mut().increase_sequence(&tx.signer_public, true).unwrap();
            }
        }
        Ok(())
    }
}

impl Stateful for Context {
    fn set_storage(&mut self, storage: ServiceRef<dyn SubStorageAccess>) {
        self.storage.replace(storage.unwrap_import().into_proxy());
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
                ExecuteError::AccountModuleError(_) => Err(()),
                ExecuteError::InvalidSequence => Err(()),
                ExecuteError::NoAccount => Err(()),
                ExecuteError::InvalidKey => Err(()),
                ExecuteError::NoToken => Err(()),
            }
        } else {
            Ok(Default::default())
        }
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        assert_eq!(transaction.tx_type(), "stamp");
        let tx: OwnTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
        tx.verify().map_err(|_| todo_fixthis)?;
        Ok(())
    }

    fn block_closed(&mut self) -> Result<Vec<Event>, CloseBlockError> {
        Ok(Vec::new())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionTransferToken {
    pub receiver: Public,

    /// There is no difference for tokens as far as the issuer is same;
    /// Thus it is enough to speicfy which token to transfer only by the issuer.
    pub issuer: H256,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Action {
    TransferToken(ActionTransferToken),
}

impl common::Action for Action {}

pub type OwnTransaction = SignedTransaction<Action>;

struct GetAccountAndSeq;

impl Service for GetAccountAndSeq {}

impl crate::sorting::GetAccountAndSeq for GetAccountAndSeq {
    fn get_account_and_seq(&self, tx: &Transaction) -> Result<(Public, u64), ()> {
        assert_eq!(tx.tx_type(), "token");
        let tx: OwnTransaction = serde_cbor::from_slice(&tx.body()).map_err(|_| ())?;
        Ok((tx.signer_public, tx.tx.seq))
    }
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            ctx: Arc::new(RwLock::new(Context {
                account: None,
                storage: None,
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "token_manager" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn TokenManager>>)
            }
            "stateful" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn Stateful>>)
            }
            "tx_owner" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn TxOwner>>)
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
            "account_manager" => {
                self.ctx.write().account.replace(import_service_from_handle(rto_context, handle));
            }
            "sub_storage_access" => {
                self.ctx.write().storage.replace(import_service_from_handle(rto_context, handle));
            }
            _ => panic!("Unsupported name in import_service() : {}", name),
        }
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
