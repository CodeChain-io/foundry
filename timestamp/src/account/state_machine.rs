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

use super::types::*;
use super::Config;
use crate::common::state_machine::{StateAccess, StateTransition};
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::context::SubStorageAccess;
use coordinator::Transaction;

pub struct GetAccount<'a> {
    pub public: &'a Public,
    pub default: bool,
}

impl<'a> StateAccess for GetAccount<'a> {
    type Outcome = Result<Account, Error>;

    fn execute(self, state: &dyn SubStorageAccess) -> Result<Account, Error> {
        let bytes = match state.get(self.public.as_ref()) {
            Some(bytes) => bytes,
            None => {
                if self.default {
                    return Ok(Default::default())
                } else {
                    return Err(Error::NoSuchAccount)
                }
            }
        };
        Ok(serde_cbor::from_slice(&bytes).map_err(|_| Error::InvalidKey)?)
    }
}

pub struct CreateAccount<'a> {
    pub public: &'a Public,
}

impl<'a> StateTransition for CreateAccount<'a> {
    type Outcome = Result<(), Error>;

    fn execute(self, state: &mut dyn SubStorageAccess) -> Result<(), Error> {
        let account = Account {
            seq: 0,
        };
        if state.has(self.public.as_ref()) {
            return Err(Error::AccountExists)
        }
        state.set(self.public.as_ref(), serde_cbor::to_vec(&account).unwrap());
        Ok(())
    }
}

/// Clone because it is recursive
#[derive(Clone)]
pub struct IncreaseSequence<'a> {
    pub public: &'a Public,
    pub default: bool,
    pub(super) config: &'a Config,
}

impl<'a> StateTransition for IncreaseSequence<'a> {
    type Outcome = Result<(), Error>;

    fn execute(self, state: &mut dyn SubStorageAccess) -> Result<(), Error> {
        let option_bytes = state.get(self.public.as_ref());
        if option_bytes.is_none() {
            if self.default {
                CreateAccount {
                    public: self.public,
                }
                .execute(state)
                .expect("Synchronization bug in AccountManager");
                self.clone().execute(state).expect("Synchronization bug in AccountManager");
                return Ok(())
            } else {
                return Err(Error::NoSuchAccount)
            }
        }
        let bytes = option_bytes.unwrap();
        let mut account: Account = serde_cbor::from_slice(&bytes).map_err(|_| Error::InvalidKey)?;
        account.seq += 1;
        state.set(self.public.as_ref(), serde_cbor::to_vec(&account).unwrap());
        Ok(())
    }
}

pub enum ExecuteError {
    InvalidMetadata,
    InvalidSign,
    InvalidFormat,
    InvalidSequence,
    AccountError(Error),
    NotAllowedHello,
}

pub struct ExecuteTransaction<'a> {
    pub tx: &'a Transaction,
    pub(super) config: &'a Config,
}

impl<'a> StateTransition for ExecuteTransaction<'a> {
    type Outcome = Result<(), ExecuteError>;

    fn execute(self, state: &mut dyn SubStorageAccess) -> Result<(), ExecuteError> {
        if self.tx.tx_type() != "account" {
            return Err(ExecuteError::InvalidMetadata)
        }

        let tx: OwnTransaction = serde_cbor::from_slice(&self.tx.body()).map_err(|_| ExecuteError::InvalidFormat)?;
        tx.verify().map_err(|_| ExecuteError::InvalidSign)?;
        if (GetAccount {
            public: &tx.signer_public,
            default: true,
        }
        .execute(state)
        .map_err(ExecuteError::AccountError)?
        .seq)
            != tx.tx.seq
        {
            return Err(ExecuteError::InvalidSequence)
        }
        if !self.config.allow_hello {
            return Err(ExecuteError::NotAllowedHello)
        }
        IncreaseSequence {
            public: &tx.signer_public,
            default: true,
            config: self.config,
        }
        .execute(state)
        .unwrap();
        Ok(())
    }
}
