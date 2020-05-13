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

use crate::context::{global, single_process_support, Config, Context, Custom, FmlConfig, PortTable};
use crate::handle::{HandlePreset, PortDispatcher};
use crate::port::Port;
use cbsb::execution::executee;
use cbsb::ipc::{intra, DefaultIpc, Ipc};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

pub fn recv<I: Ipc, T: serde::de::DeserializeOwned>(ctx: &executee::Context<I>) -> T {
    serde_cbor::from_slice(&ctx.ipc.as_ref().unwrap().recv(None).unwrap()).unwrap()
}

pub fn send<I: Ipc, T: serde::Serialize>(ctx: &executee::Context<I>, data: &T) {
    ctx.ipc.as_ref().unwrap().send(&serde_cbor::to_vec(data).unwrap());
}

fn create_port(
    ipc_type: Vec<u8>,
    ipc_config: Vec<u8>,
    dispatcher: Arc<PortDispatcher>,
    instance_key: single_process_support::InstanceKey,
    config_fml: &FmlConfig,
) -> Port {
    let ipc_type: String = serde_cbor::from_slice(&ipc_type).unwrap();

    if ipc_type == "DomainSocket" {
        let ipc = DefaultIpc::new(ipc_config);
        let (send, recv) = ipc.split();
        Port::new(send, recv, dispatcher, instance_key, config_fml)
    } else if ipc_type == "Intra" {
        let ipc = intra::Intra::new(ipc_config);
        let (send, recv) = ipc.split();
        Port::new(send, recv, dispatcher, instance_key, config_fml)
    } else {
        panic!("Invalid port creation request");
    }
}

type DebugFunction = Box<dyn Fn(Vec<u8>) -> Vec<u8>>;

pub fn run_control_loop<I: Ipc, C: Custom, H: HandlePreset>(
    args: Vec<String>,
    context_setter: Box<dyn Fn(Context<C>) -> ()>,
    debug: Option<DebugFunction>,
) {
    let ctx = executee::start::<I>(args);

    let handle_descriptor: crate::handle::id::IdMap = recv(&ctx);
    let config: Config = recv(&ctx);
    let config_fml: FmlConfig = recv(&ctx);
    let _id = config.id.clone();
    let instance_key: single_process_support::InstanceKey = config.key;
    // set instance key also of this main thread.
    single_process_support::set_key(instance_key);
    super::handle::id::setup_identifiers(instance_key, &handle_descriptor);
    let custom = C::new(&config);
    let ports: Arc<RwLock<PortTable>> = Arc::new(RwLock::new(PortTable {
        map: HashMap::new(),
        no_drop: false,
    }));
    let global_context = Context::new(ports.clone(), config, config_fml.clone(), custom);
    global::set(global_context.ports.clone());
    context_setter(global_context);
    loop {
        let message: String = recv(&ctx);
        if message == "link" {
            let (port_id, counter_port_id, port_config, ipc_type, ipc_config) = recv(&ctx);
            let dispather = Arc::new(PortDispatcher::new(port_id, 128));
            let mut port_table = ports.write().unwrap();

            let old = port_table.map.insert(
                port_id,
                (port_config, counter_port_id, create_port(ipc_type, ipc_config, dispather, instance_key, &config_fml)),
            );
            // we assert before drop old to avoid (hard-to-debug) blocking.
            assert!(old.is_none(), "You must unlink first to link an existing port");
        } else if message == "unlink" {
            let (port_id,) = recv(&ctx);
            let mut port_table = ports.write().unwrap();
            port_table.map.remove(&port_id).unwrap();
        } else if message == "terminate" {
            break
        } else if message == "handle_export" {
            // export a default, preset handles for a specific port
            send(&ctx, &H::export());
        } else if message == "handle_import" {
            // import a default, preset handles for a specific port
            let (handles,) = recv(&ctx);
            H::import(handles);
        } else if message == "debug" {
            // temporarily give the execution flow to module, and the module
            // may do whatever it wants but must return a result to report back
            // to host.
            let (args,) = recv(&ctx);
            let result = debug.as_ref().expect("You didn't provide any debug routine")(args);
            send(&ctx, &result);
        } else {
            panic!("Unexpected message: {}", message)
        }
        send(&ctx, &"done".to_owned());
    }
    ctx.terminate();
}
