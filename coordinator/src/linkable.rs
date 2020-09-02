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

use crate::Services;
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
use std::mem;
use std::sync::Arc;

pub(crate) static HOST_PATH: &str = "$";
static TX_SERVICE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^@tx/([^/]+)/([^/]+)$").unwrap());
static SERVICE_RE: Lazy<Regex> = Lazy::new(|| Regex::new(r"^([^/]+)/([^/]+)$").unwrap());

static SERVICES: Lazy<Mutex<Services>> = Lazy::new(|| Mutex::new(Services::new()));

#[distributed_slice(MODULE_INITS)]
fn init() {
    executor::add_function_pool(
        HOST_PATH.to_owned(),
        Arc::new(foundry_module_rt::start::<<SingleProcess as ExecutionScheme>::Ipc, HostModule>),
    );
}

pub(super) fn services() -> Services {
    mem::take(&mut SERVICES.lock())
}

#[derive(Default)]
struct HostModule;

impl UserModule for HostModule {
    fn new(_arg: &[u8]) -> Self {
        HostModule::default()
    }

    fn prepare_service_to_export(&mut self, _ctor_name: &str, _ctor_arg: &[u8]) -> Skeleton {
        panic!("Nothing exported yet")
    }

    fn import_service(&mut self, rto_context: &Context, name: &str, handle: HandleToExchange) {
        let mut services = SERVICES.lock();

        if let Some(cap) = TX_SERVICE_RE.captures(name) {
            if &cap[2] == "tx-owner" {
                services.tx_owner.insert(cap[1].to_owned(), import_service_from_handle(rto_context, handle));
                return
            }
            panic!("Unknown import: {}", name)
        }
        if let Some(cap) = SERVICE_RE.captures(name) {
            let module = &cap[2];
            match &cap[1] {
                "init-genesis" => {
                    services.init_genesis.push((module.to_owned(), import_service_from_handle(rto_context, handle)));
                }
                "init-chain" => {
                    services.init_chain = import_service_from_handle(rto_context, handle);
                }
                "update-chain" => {
                    services.update_chain = import_service_from_handle(rto_context, handle);
                }
                "stateful" => {
                    services.stateful.lock().push((module.to_owned(), import_service_from_handle(rto_context, handle)));
                }
                "tx-sorter" => {
                    services.tx_sorter = import_service_from_handle(rto_context, handle);
                }
                "handle-graphql-request" => {
                    services.handle_graphqls.push((module.to_owned(), import_service_from_handle(rto_context, handle)));
                }
                _ => panic!("Unknown import: {}", name),
            }
            return
        }
        panic!("Unknown import: {}", name)
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
