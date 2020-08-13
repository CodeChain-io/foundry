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
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::module::*;
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{Context as RtoContext, Service};
use std::collections::HashMap;
use std::sync::Arc;

pub struct Context {
    pub tx_owners: HashMap<String, Box<dyn TxOwner>>,
    pub init_genesises: HashMap<String, Box<dyn InitGenesis>>,
    pub statefuls: HashMap<String, Box<dyn Stateful>>,
    pub init_chain: Option<Box<dyn InitChain>>,
    pub update_chain: Option<Box<dyn UpdateChain>>,
    pub tx_sorter: Option<Box<dyn TxSorter>>,
}

impl Service for Context {}

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
            })),
        }
    }

    fn prepare_service_to_export(&mut self, _ctor_name: &str, _ctor_arg: &[u8]) -> Skeleton {
        panic!("Coordinator doesn't export anything!")
    }

    fn import_service(&mut self, rto_context: &RtoContext, name: &str, handle: HandleToExchange) {
        let tokens: Vec<&str> = name.split('/').collect();
        assert_eq!(tokens.len(), 2);
        let exporter_module = tokens[0];
        let name = tokens[1];
        match name {
            "tx_owner" => assert!(self
                .ctx
                .write()
                .tx_owners
                .insert(exporter_module.to_owned(), import_service_from_handle(rto_context, handle))
                .is_none()),
            "init_genesis" => assert!(self
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
            "init_chain" => {
                assert!(self.ctx.write().init_chain.replace(import_service_from_handle(rto_context, handle)).is_none())
            }
            "update_chain" => assert!(self
                .ctx
                .write()
                .update_chain
                .replace(import_service_from_handle(rto_context, handle))
                .is_none()),
            "tx_sorter" => {
                assert!(self.ctx.write().tx_sorter.replace(import_service_from_handle(rto_context, handle)).is_none())
            }
            _ => panic!("Unsupported name in import_service() : {}", name),
        }
    }

    fn debug(&mut self, arg: &[u8]) -> Vec<u8> {
        let scenario: String = serde_cbor::from_slice(arg).unwrap();
        match scenario.as_str() {
            "simple1" => scenarios::simple1(self.ctx.as_ref()),
            "multiple" => scenarios::multiple(self.ctx.as_ref()),
            _ => panic!(),
        }
        Vec::new()
    }
}
