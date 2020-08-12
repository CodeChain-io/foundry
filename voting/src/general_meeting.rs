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

use ckey::Ed25519Public as Public;
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::types::*;
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{Context as RtoContext, Service, ServiceRef};
use std::sync::Arc;

const ADMIN_STATE_KEY: &str = "admin";

struct Context {
    pub storage: Option<Box<dyn SubStorageAccess>>,
}

impl Context {
    fn storage(&self) -> &dyn SubStorageAccess {
        self.storage.as_ref().unwrap().as_ref()
    }

    fn storage_mut(&mut self) -> &mut dyn SubStorageAccess {
        self.storage.as_mut().unwrap().as_mut()
    }

    fn admin(&self) -> Public {
        let bytes = self
            .storage()
            .get(ADMIN_STATE_KEY.as_bytes())
            .expect("GeneralMeeting module set the admin in the genesis state");
        serde_cbor::from_slice(&bytes).expect("Admin key is saved in the GeneralMeeting module")
    }
}

impl Service for Context {}

impl Stateful for Context {
    fn set_storage(&mut self, storage: ServiceRef<dyn SubStorageAccess>) {
        self.storage.replace(storage.unwrap_import().into_remote());
    }
}

impl InitGenesis for Context {
    fn begin_genesis(&mut self) {}

    fn init_genesis(&mut self, config: &[u8]) {
        let admin: Public = serde_cbor::from_slice(&config).unwrap();
        self.storage_mut().set(ADMIN_STATE_KEY.as_bytes(), serde_cbor::to_vec(&admin).unwrap());
    }

    fn end_genesis(&mut self) {}
}

impl TxOwner for Context {
    fn block_opened(&mut self, _: &Header) -> Result<(), HeaderError> {
        Ok(())
    }

    fn execute_transaction(&mut self, transaction: &Transaction) -> Result<TransactionOutcome, ()> {
        unimplemented!();
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), coordinator::types::ErrorCode> {
        unimplemented!();
    }

    fn block_closed(&mut self) -> Result<Vec<Event>, CloseBlockError> {
        Ok(Vec::new())
    }
}

pub struct Module {
    ctx: Arc<RwLock<Context>>,
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
            "tx_owner" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn TxOwner>>)
            }
            "init_genesis" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn InitGenesis>>)
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
