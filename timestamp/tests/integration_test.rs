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

extern crate codechain_module as cmodule;
extern crate codechain_timestamp as timestamp;
extern crate foundry_process_sandbox as fproc_sndbx;

mod common;

use cmodule::impls::process::*;
use cmodule::link::*;
use cmodule::sandbox::*;
use common::mock_coordinator::MockCoordinator;
use foundry_module_rt::UserModule;
use fproc_sndbx::execution::executor;
use fproc_sndbx::ipc::generate_random_name;
use std::collections::HashMap;
use std::sync::Arc;
use timestamp::account::Module as AccountModule;
use timestamp::sorting::Module as SortingModule;
use timestamp::staking::Module as StakingModule;
use timestamp::stamp::Module as StampModule;
use timestamp::token::Module as TokenModule;

fn execute_module<M: UserModule + 'static>(args: Vec<String>) {
    foundry_module_rt::start::<<SingleProcess as ExecutionScheme>::Ipc, M>(args);
}

fn load_sandbox<M: UserModule + 'static>(
    sandboxer: &ProcessSandboxer<SingleProcess>,
    init: &dyn erased_serde::Serialize,
    exports: &[(&str, &dyn erased_serde::Serialize)],
) -> Box<dyn Sandbox> {
    let name = generate_random_name();
    executor::add_function_pool(name.clone(), Arc::new(execute_module::<M>));
    let name = std::path::PathBuf::from(name);
    sandboxer.load(&name, init, exports).unwrap()
}

/// Map of (Exporter, Importer) to (exporting ctor, importing name)
type LinkTable = HashMap<&'static str, Vec<(&'static str, &'static str, &'static str)>>;

fn generate_link_table() -> LinkTable {
    let mut map = HashMap::new();

    map.insert("account", vec![
        ("token", "account_manager", "account_manager"),
        ("stamp", "account_manager", "account_manager"),
        ("sorting", "account_manager", "account_manager"),
        ("coordinator", "stateful", "account/stateful"),
    ]);

    map.insert("staking", vec![
        ("coordinator", "init_genesis", "staking/init_genesis"),
        ("coordinator", "init_chain", "staking/init_chain"),
        ("coordinator", "update_chain", "staking/update_chain"),
    ]);

    map.insert("stamp", vec![
        ("sorting", "get_account_and_seq", "stamp/get_account_and_seq"),
        ("coordinator", "tx_owner", "stamp/tx_owner"),
        ("coordinator", "init_genesis", "stamp/init_genesis"),
    ]);

    map.insert("token", vec![
        ("staking", "token_manager", "token_manager"),
        ("stamp", "token_manager", "token_manager"),
        ("sorting", "get_account_and_seq", "token/get_account_and_seq"),
        ("coordinator", "tx_owner", "token/tx_owner"),
        ("coordinator", "stateful", "token/stateful"),
    ]);

    map.insert("sorting", vec![("coordinator", "tx_sorter", "sorting/tx_sorter")]);

    map.insert("coordinator", vec![]);

    map
}

pub fn setup() -> HashMap<&'static str, Box<dyn Sandbox>> {
    let sandboxer = ProcessSandboxer::<SingleProcess>::new();

    let link_table = generate_link_table();
    let mut modules = HashMap::new();

    modules.insert("account", {
        let exports: Vec<(&str, &dyn erased_serde::Serialize)> = link_table
            .get("account")
            .unwrap()
            .iter()
            .map(|(_, ctor, _)| (*ctor, &"unused" as &dyn erased_serde::Serialize))
            .collect();
        load_sandbox::<AccountModule>(&sandboxer, &"unused", &exports)
    });

    modules.insert("staking", {
        let exports: Vec<(&str, &dyn erased_serde::Serialize)> = link_table
            .get("staking")
            .unwrap()
            .iter()
            .map(|(_, ctor, _)| (*ctor, &"unused" as &dyn erased_serde::Serialize))
            .collect();
        load_sandbox::<StakingModule>(&sandboxer, &"unused", &exports)
    });

    modules.insert("stamp", {
        let exports: Vec<(&str, &dyn erased_serde::Serialize)> = link_table
            .get("stamp")
            .unwrap()
            .iter()
            .map(|(_, ctor, _)| (*ctor, &"unused" as &dyn erased_serde::Serialize))
            .collect();
        load_sandbox::<StampModule>(&sandboxer, &"unused", &exports)
    });

    modules.insert("token", {
        let exports: Vec<(&str, &dyn erased_serde::Serialize)> = link_table
            .get("token")
            .unwrap()
            .iter()
            .map(|(_, ctor, _)| (*ctor, &"unused" as &dyn erased_serde::Serialize))
            .collect();
        load_sandbox::<TokenModule>(&sandboxer, &"unused", &exports)
    });

    modules.insert("sorting", {
        let exports: Vec<(&str, &dyn erased_serde::Serialize)> = link_table
            .get("sorting")
            .unwrap()
            .iter()
            .map(|(_, ctor, _)| (*ctor, &"unused" as &dyn erased_serde::Serialize))
            .collect();
        load_sandbox::<SortingModule>(&sandboxer, &"unused", &exports)
    });

    modules.insert("coordinator", {
        let exports: Vec<(&str, &dyn erased_serde::Serialize)> = link_table
            .get("coordinator")
            .unwrap()
            .iter()
            .map(|(_, ctor, _)| (*ctor, &"unused" as &dyn erased_serde::Serialize))
            .collect();
        load_sandbox::<MockCoordinator>(&sandboxer, &"unused", &exports)
    });

    let module_names: Vec<&'static str> = modules.keys().cloned().collect();

    // N * (N-1)
    let mut export_and_import = HashMap::new();
    for &name1 in module_names.iter() {
        for &name2 in module_names.iter() {
            if name1 == name2 {
                continue
            }
            export_and_import.insert((name1, name2), (Vec::new(), Vec::new()));
        }
    }

    // N * (N-1) / 2
    let mut ports = HashMap::new();
    for &name1 in module_names.iter() {
        for &name2 in module_names.iter() {
            if name1 == name2 {
                continue
            }
            if name1 > name2 {
                continue
            }
            let port1 = modules.get_mut(name1).unwrap().new_port();
            let port2 = modules.get_mut(name2).unwrap().new_port();

            ports.insert((name1, name2), (port1, port2));
        }
    }

    for (exporter, exchanges) in link_table.iter() {
        for (index, (importer, _, import_name)) in exchanges.iter().enumerate() {
            let (exports, imports) = export_and_import.get_mut(&(exporter, importer)).unwrap();
            exports.push(index);
            imports.push(*import_name);
        }
    }

    // N * (N-1)
    for &exporter in module_names.iter() {
        for &importer in module_names.iter() {
            if exporter == importer {
                continue
            }
            let (exporter_port, importer_port) = if exporter < importer {
                let (p1, p2) = ports.get_mut(&(exporter, importer)).unwrap();
                (p1, p2)
            } else {
                let (p1, p2) = ports.get_mut(&(importer, exporter)).unwrap();
                (p2, p1)
            };
            let (exports, imports) = export_and_import.get(&(exporter, importer)).unwrap();
            exporter_port.export(exports);
            importer_port.import(imports);
        }
    }

    // N * (N-1) / 2
    for (port1, port2) in ports.values_mut() {
        let linker = ProcessLinker::<SingleProcess>::new();
        linker.link(port1.as_mut(), port2.as_mut()).unwrap();
    }

    for module in modules.values_mut() {
        module.seal();
    }

    modules
}

#[test]
fn simple1() {
    let mut modules = setup();
    let coordinator = modules.get_mut("coordinator").unwrap().as_mut();
    coordinator.debug(&serde_cbor::to_vec(&"simple1").unwrap());
}

#[test]
fn multiple() {
    let mut modules = setup();
    let coordinator = modules.get_mut("coordinator").unwrap().as_mut();
    coordinator.debug(&serde_cbor::to_vec(&"multiple").unwrap());
}
