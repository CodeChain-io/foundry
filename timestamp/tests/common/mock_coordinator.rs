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

use super::scenarios;
use ccrypto::blake256;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use primitives::H256;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{Context as RtoContext, Service, ServiceRef};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct MockDb {
    map: HashMap<H256, Vec<u8>>,
}

impl Service for MockDb {}

impl SubStorageAccess for MockDb {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.map.get(&blake256(key)).cloned()
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        self.map.insert(blake256(key), value);
    }

    fn remove(&mut self, key: &[u8]) {
        self.map.remove(&blake256(key));
    }

    fn has(&self, key: &[u8]) -> bool {
        self.map.get(&blake256(key)).is_some()
    }

    fn create_checkpoint(&mut self) {
        unimplemented!()
    }

    fn discard_checkpoint(&mut self) {
        unimplemented!()
    }

    fn revert_to_the_checkpoint(&mut self) {
        unimplemented!()
    }
}

pub struct Context {
    pub tx_owners: HashMap<String, Box<dyn TxOwner>>,
    pub init_genesises: HashMap<String, Box<dyn InitGenesis>>,
    pub statefuls: HashMap<String, Box<dyn Stateful>>,
    pub init_chain: Option<Box<dyn InitChain>>,
    pub update_chain: Option<Box<dyn UpdateChain>>,
    pub tx_sorter: Option<Box<dyn TxSorter>>,
    pub handle_graphqls: HashMap<String, Box<dyn HandleGraphQlRequest>>,

    sub_storages: HashMap<String, Arc<RwLock<dyn SubStorageAccess>>>,
}

impl Service for Context {}

impl Context {
    pub fn get_storage(&mut self, name: String) -> Arc<RwLock<dyn SubStorageAccess>> {
        if let Some(x) = self.sub_storages.get(&name) {
            Arc::clone(x)
        } else {
            let x = Arc::new(RwLock::new(MockDb::default())) as Arc<RwLock<dyn SubStorageAccess>>;
            self.sub_storages.insert(name, Arc::clone(&x));
            x
        }
    }
}

struct GetStorageImpl {
    storage: Arc<RwLock<dyn SubStorageAccess>>,
}

impl Service for GetStorageImpl {}

impl GetStorage for GetStorageImpl {
    fn get_storage(&self, block_height: Option<u64>) -> Option<ServiceRef<dyn SubStorageAccess>> {
        assert!(block_height.is_none());
        Some(ServiceRef::create_export(Arc::clone(&self.storage)))
    }
}

pub struct MockCoordinator {
    ctx: Arc<RwLock<Context>>,
}

impl UserModule for MockCoordinator {
    fn new(_arg: &[u8]) -> Self {
        MockCoordinator {
            ctx: Arc::new(RwLock::new(Context {
                tx_owners: Default::default(),
                init_genesises: Default::default(),
                statefuls: Default::default(),
                init_chain: Default::default(),
                update_chain: Default::default(),
                tx_sorter: Default::default(),
                handle_graphqls: Default::default(),
                sub_storages: Default::default(),
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, _ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "get-storage" => {
                // TODO: use ctor_arg for this
                let module_name = "account".to_owned();
                Skeleton::new(Box::new(GetStorageImpl {
                    storage: self.ctx.write().get_storage(module_name),
                }) as Box<dyn GetStorage>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(&mut self, rto_context: &RtoContext, name: &str, handle: HandleToExchange) {
        let tokens: Vec<&str> = name.split('/').collect();
        assert_eq!(tokens.len(), 2);
        let exporter_module = tokens[0];
        let name = tokens[1];
        match name {
            "tx-owner" => assert!(self
                .ctx
                .write()
                .tx_owners
                .insert(exporter_module.to_owned(), import_service_from_handle(rto_context, handle))
                .is_none()),
            "init-genesis" => assert!(self
                .ctx
                .write()
                .init_genesises
                .insert(exporter_module.to_owned(), import_service_from_handle(rto_context, handle))
                .is_none()),
            "stateful" => assert!(self
                .ctx
                .write()
                .statefuls
                .insert(exporter_module.to_owned(), import_service_from_handle(rto_context, handle))
                .is_none()),
            "init-chain" => {
                assert!(self.ctx.write().init_chain.replace(import_service_from_handle(rto_context, handle)).is_none())
            }
            "update-chain" => assert!(self
                .ctx
                .write()
                .update_chain
                .replace(import_service_from_handle(rto_context, handle))
                .is_none()),
            "tx-sorter" => {
                assert!(self.ctx.write().tx_sorter.replace(import_service_from_handle(rto_context, handle)).is_none())
            }
            "handle-graphql-request" => assert!(self
                .ctx
                .write()
                .handle_graphqls
                .insert(exporter_module.to_owned(), import_service_from_handle(rto_context, handle))
                .is_none()),
            _ => panic!("Unsupported name in import_service() : {}", name),
        }
    }

    fn debug(&mut self, arg: &[u8]) -> Vec<u8> {
        let scenario: String = serde_cbor::from_slice(arg).unwrap();
        match scenario.as_str() {
            "simple1" => scenarios::simple1(self.ctx.as_ref()),
            "multiple" => scenarios::multiple(self.ctx.as_ref()),
            "sort" => scenarios::sort(self.ctx.as_ref()),
            "query" => scenarios::query(self.ctx.as_ref()),
            _ => panic!(),
        }
        Vec::new()
    }
}
