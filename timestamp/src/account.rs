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
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{service, Context as RtoContext, Service, ServiceRef};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

pub struct Context {
    pub storage: Option<Box<dyn SubStorageAccess>>,
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
        self.storage.replace(storage.unwrap_import().into_remote());
    }
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            ctx: Arc::new(RwLock::new(Context {
                storage: None,
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
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
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(
        &mut self,
        rto_context: &RtoContext,
        _exporter_module: &str,
        name: &str,
        handle: HandleToExchange,
    ) {
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
