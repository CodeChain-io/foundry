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

use crate::module::{InitChain, InitGenesis, Stateful, TxOwner, TxSorter, UpdateChain};
use crate::Inner;
use cmodule::impls::process::{ExecutionScheme, SingleProcess};
use cmodule::MODULE_INITS;

use foundry_module_rt::UserModule;
use foundry_process_sandbox::execution::executor;
use linkme::distributed_slice;
use once_cell::sync::Lazy;
use parking_lot::Mutex;
use regex::Regex;
use remote_trait_object::raw_exchange::Skeleton;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange};
use remote_trait_object::Context;

use std::collections::HashMap;
use std::sync::Arc;

pub(crate) static HOST_PATH: &str = "$";
static TX_SERVICE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^@tx/([^/])/([^/])$").unwrap());
static SERVICE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^/])/([^/])$").unwrap());

static SERVICES: Lazy<Mutex<Services>> = Lazy::new(|| Mutex::new(Services::default()));

#[distributed_slice(MODULE_INITS)]
fn init() {
    executor::add_function_pool(
        HOST_PATH.to_owned(),
        Arc::new(foundry_module_rt::start::<<SingleProcess as ExecutionScheme>::Ipc, HostModule>),
    );
}

pub(super) fn inner() -> Inner {
    SERVICES.lock().inner()
}

#[derive(Default)]
struct HostModule;

#[derive(Default)]
pub(crate) struct Services {
    tx_owners: HashMap<String, Box<dyn TxOwner>>,
    init_genesis: Vec<(String, Box<dyn InitGenesis>)>,
    init_chain: Option<Box<dyn InitChain>>,
    update_chain: Option<Box<dyn UpdateChain>>,
    stateful: Vec<(String, Box<dyn Stateful>)>,
    tx_sorter: Option<Box<dyn TxSorter>>,
}

impl Services {
    pub(super) fn inner(&mut self) -> Inner {
        let mut inner = Inner::default();

        inner.tx_owner = self.tx_owners.drain().collect();
        inner.init_genesis = self.init_genesis.drain(..).collect();
        if let Some(init_chain) = self.init_chain.take() {
            inner.init_chain = init_chain;
        }
        if let Some(update_chain) = self.update_chain.take() {
            inner.update_chain = update_chain;
        }
        inner.stateful = self.stateful.drain(..).collect();
        if let Some(tx_sorter) = self.tx_sorter.take() {
            inner.tx_sorter = tx_sorter;
        }

        inner
    }
}

impl UserModule for HostModule {
    fn new(_arg: &[u8]) -> Self {
        // Initialize here to repeat creation of Coordinator upon updates
        *SERVICES.lock() = Services::default();
        HostModule::default()
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        panic!("Nothing exported yet")
    }

    fn import_service(&mut self, rto_context: &Context, _exporter_module: &str, name: &str, handle: HandleToExchange) {
        let mut services = SERVICES.lock();

        if let Some(cap) = TX_SERVICE_RE.captures(name) {
            if &cap[2] == "tx-owner" {
                services.tx_owners.insert(cap[1].to_owned(), import_service_from_handle(rto_context, handle));
                return
            }
        }
        if let Some(cap) = SERVICE_RE.captures(name) {
            let module = &cap[2];
            match &cap[1] {
                "init-genesis" => {
                    services.init_genesis.push((module.to_owned(), import_service_from_handle(rto_context, handle)));
                }
                "init-chain" => {
                    services.init_chain.replace(import_service_from_handle(rto_context, handle));
                }
                "update-chain" => {
                    services.update_chain.replace(import_service_from_handle(rto_context, handle));
                }
                "stateful" => {
                    services.stateful.push((module.to_owned(), import_service_from_handle(rto_context, handle)));
                }
                "tx-sorter" => {
                    services.tx_sorter.replace(import_service_from_handle(rto_context, handle));
                }
                _ => {}
            }
        }
        panic!("Unknown import: {}", name)
    }

    fn debug(&mut self, arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
