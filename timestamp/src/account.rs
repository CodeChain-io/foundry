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

pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Context {
    pub storage: Arc<RwLock<dyn SubStorageAccess>>,
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

pub trait AccountManager: Send + Sync {
    fn create_account(&self, public: &Public) -> Result<(), Error>;

    fn get_sequence(&self, public: &Public, default: bool) -> Result<u64, Error>;

    fn increase_sequence(&self, public: &Public, default: bool) -> Result<(), Error>;
}

impl AccountManager for Context {
    fn create_account(&self, public: &Public) -> Result<(), Error> {
        let account = Account {
            seq: 0,
        };
        if self.storage.read().has(public) {
            return Err(Error::AccountExists)
        }
        self.storage.write().set(public, serde_cbor::to_vec(&account).unwrap());
        Ok(())
    }

    fn get_sequence(&self, public: &Public, default: bool) -> Result<u64, Error> {
        let bytes = match self.storage.read().get(public) {
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

    fn increase_sequence(&self, public: &Public, default: bool) -> Result<(), Error> {
        let option_bytes = self.storage.read().get(public);
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
        self.storage.write().set(public, serde_cbor::to_vec(&account).unwrap());
        Ok(())
    }
}

impl Stateful for Context {
    fn set_storage(&mut self, _storage: Box<dyn SubStorageAccess>) {
        unimplemented!()
    }
}
