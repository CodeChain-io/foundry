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

use cbsb::execution::executor::{self, Executor};
use cbsb::ipc::generate_random_name;
use cbsb::ipc::Ipc;
use fml::*;
use std::collections::HashMap;

const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100_000);
pub const SERVER_THREADS: usize = 16;

pub struct FmlModule<I: Ipc, E: Executor> {
    ctx: Option<executor::Context<I, E>>,
    config: Config,
}

impl<I: Ipc, E: Executor> FmlModule<I, E> {
    pub fn new(ctx: executor::Context<I, E>, trait_map: HashMap<String, TraitId>, id: String, args: Vec<u8>) -> Self {
        let descriptor = IdMap {
            trait_map,
            method_map: HashMap::new(),
        };
        let config = Config {
            kind: generate_random_name(),
            id,
            key: super::key::create_instance(),
            args,
        };
        let config_fml = FmlConfig {
            server_threads: SERVER_THREADS,
            call_slots: 128,
        };

        //let module_name = generate_random_name();
        //executor::add_function_pool(module_name.clone(), fun);

        let result = FmlModule {
            ctx: Some(ctx),
            config: config.clone(),
        };
        result.send(&descriptor);
        result.send(&config);
        result.send(&config_fml);
        result
    }

    fn recv<T: serde::de::DeserializeOwned>(&self) -> T {
        serde_cbor::from_slice(&self.ctx.as_ref().unwrap().ipc.recv(Some(TIMEOUT)).unwrap()).unwrap()
    }

    fn send<T: serde::Serialize>(&self, data: &T) {
        self.ctx.as_ref().unwrap().ipc.send(&serde_cbor::to_vec(data).unwrap());
    }

    fn done_ack(&self) {
        assert_eq!(
            {
                let x: String = self.recv();
                x
            },
            "done"
        );
    }

    pub fn debug(&self, arg: Vec<u8>) -> Vec<u8> {
        self.send(&"debug");
        self.send(&(arg,));
        let result: Vec<u8> = self.recv();
        self.done_ack();
        result
    }
}

impl<I: Ipc, E: Executor> Drop for FmlModule<I, E> {
    fn drop(&mut self) {
        self.send(&"terminate");
        self.ctx.take().unwrap().terminate();
        super::key::return_instance(self.config.key);
    }
}

pub type Modules<I, E> = HashMap<String, FmlModule<I, E>>;

pub trait LinkMessage {
    fn link_message() -> &'static str;
}

impl LinkMessage for cbsb::ipc::DefaultIpc {
    fn link_message() -> &'static str {
        "DomainSocket"
    }
}

impl LinkMessage for cbsb::ipc::intra::Intra {
    fn link_message() -> &'static str {
        "Intra"
    }
}

/// Link all modules once. For the test, one link per one pair is enough.
pub fn link_all<I: Ipc + LinkMessage, E: Executor>(modules: &Modules<I, E>) {
    let mut port_count = HashMap::<&String, usize>::new();
    for k in modules.keys() {
        port_count.insert(k, 0);
    }

    for (name1, module1) in modules {
        for (name2, module2) in modules {
            if name1 == name2 {
                continue
            }

            let port1 = port_count.get(name1).unwrap();
            let port2 = port_count.get(name2).unwrap();

            // already established
            if *port1 == port_count.len() - 1 || *port2 == port_count.len() - 1 {
                continue
            }

            let (ipc_config1, ipc_config2) = <I as cbsb::ipc::Ipc>::arguments_for_both_ends();
            let link_message = <I as LinkMessage>::link_message();

            module1.send(&"link");
            module1.send(&(
                *port1,
                *port2,
                module2.config.clone(),
                serde_cbor::to_vec(&link_message).unwrap(),
                ipc_config1,
            ));

            module2.send(&"link");
            module2.send(&(
                *port2,
                *port1,
                module1.config.clone(),
                serde_cbor::to_vec(&link_message).unwrap(),
                ipc_config2,
            ));

            module1.done_ack();
            module2.done_ack();

            *port_count.get_mut(name1).unwrap() += 1;
            *port_count.get_mut(name2).unwrap() += 1;
        }
    }
}

/// Perform the initial handle exchange for all modules
pub fn exchange<I: Ipc, E: Executor>(modules: &Modules<I, E>) {
    for module in modules.values() {
        module.send(&"handle_export");
        let exchange_list: Vec<HandleExchange> = module.recv();
        module.done_ack();

        for exchange in &exchange_list {
            let importer = modules.get(&exchange.importer).unwrap();
            importer.send(&"handle_import");
            importer.send(&(&exchange,));
            importer.done_ack();
        }
    }
}
