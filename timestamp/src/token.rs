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
use parking_lot::RwLock;
use primitives::H256;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Context {
    pub account: RwLock<Arc<dyn AccountManager>>,
    pub storage: RwLock<Arc<dyn SubStorageAccess>>,
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

#[derive(Debug)]
pub enum Error {
    NoSuchAccount,
    InvalidKey,
}

pub trait TokenManager: Send + Sync {
    fn get_account(&self, public: Public) -> Result<Account, Error>;
    fn issue_token(&self, issuer: H256, receiver: Public) -> Result<(), Error>;
}

impl TokenManager for Context {
    fn get_account(&self, public: Public) -> Result<Account, Error> {
        let x = self.storage.read().get(&Self::get_key(public)).ok_or_else(|| Error::NoSuchAccount)?;
        Ok(serde_cbor::from_slice(&x).map_err(|_| Error::InvalidKey)?)
    }

    fn issue_token(&self, issuer: H256, receiver: Public) -> Result<(), Error> {
        let mut account = self.get_account_or_default(receiver).map_err(|_| Error::InvalidKey)?;
        account.tokens.push(Token {
            issuer,
        });
        self.set_account(receiver, &account);
        Ok(())
    }
}

impl BlockOpen for Context {
    fn block_opened(&self, storage: Box<dyn SubStorageAccess>) -> Result<(), HeaderError> {
        *self.storage.write() = Arc::from(storage);
        Ok(())
    }
}

impl BlockClosed for Context {
    fn block_closed(&self) -> Result<BlockOutcome, CloseBlockError> {
        Ok(BlockOutcome {
            updated_consensus_params: None,
            updated_validator_set: None,
            events: Vec::new(),
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
    fn get_key(key: Public) -> H256 {
        blake256(&{
            let mut v = serde_cbor::to_vec(&key).unwrap();
            v.extend_from_slice(b"Token-Module");
            v
        } as &[u8])
    }

    fn get_account_or_default(&self, key: Public) -> Result<Account, ()> {
        if let Some(x) = self.storage.read().get(&Self::get_key(key)) {
            Ok(serde_cbor::from_slice(&x).map_err(|_| ())?)
        } else {
            Ok(Account {
                tokens: Vec::new(),
            })
        }
    }

    /// set_account() must not fail
    fn set_account(&self, key: Public, account: &Account) {
        self.storage.read().set(&Self::get_key(key), serde_cbor::to_vec(account).unwrap());
    }

    fn excute_tx(&self, transaction: &Transaction) -> Result<(), ExecuteError> {
        if transaction.tx_type() != "Token" {
            return Err(ExecuteError::InvalidMetadata)
        }
        let tx: OwnTransaction =
            serde_cbor::from_slice(&transaction.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;
        if self.account.read().get_sequence(&tx.signer_public, true).map_err(ExecuteError::AccountModuleError)?
            != tx.tx.seq
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
                        .storage
                        .read()
                        .get(&Self::get_key(tx.signer_public))
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
                    self.get_account_or_default(receiver).map_err(|_| ExecuteError::InvalidKey)?;

                // From now on, it will actually mutate the state and must not fail
                // to keep the consistency of the state.
                // This issue might be handled by the coordinator and should be decided in detail.

                recipient_account.tokens.push(token);
                self.set_account(tx.signer_public, &sender_account);
                self.set_account(receiver, &recipient_account);
                self.account.read().increase_sequence(&tx.signer_public, true).unwrap();
            }
        }
        Ok(())
    }
}

impl TxOwner for Context {
    fn execute_transaction(&self, transaction: &Transaction) -> Result<TransactionExecutionOutcome, ()> {
        if let Err(error) = self.excute_tx(transaction) {
            match error {
                ExecuteError::InvalidMetadata => Err(()),
                ExecuteError::InvalidSign => Err(()),
                ExecuteError::InvalidFormat => Err(()),
                ExecuteError::AccountModuleError(_) => Err(()),
                ExecuteError::InvalidSequence => Err(()),
                ExecuteError::NoAccount => Err(()),
                ExecuteError::InvalidKey => Err(()),
                ExecuteError::NoToken => Ok(Default::default()), // Don't reject; just accept though it fails to mutate something.
            }
        } else {
            Ok(Default::default())
        }
    }

    fn propose_transaction(&self, _transaction: &TransactionWithMetadata) -> bool {
        unimplemented!()
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        let todo_fixthis: coordinator::types::ErrorCode = 3;
        assert_eq!(transaction.tx_type(), "Stamp");
        let tx: OwnTransaction = serde_cbor::from_slice(&transaction.body()).map_err(|_| todo_fixthis)?;
        tx.verify().map_err(|_| todo_fixthis)?;
        Ok(())
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
