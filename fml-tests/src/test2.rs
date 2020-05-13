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

use super::module::*;
use cbsb::execution::executor::{self, Executor};
use cbsb::ipc::Ipc;
use std::collections::HashMap;
use std::sync::{Arc, Barrier};
use std::thread;

pub fn run<I: Ipc + 'static + LinkMessage, E: Executor + 'static>(mod_relayer_path: &str, mod_scheduler_path: &str) {
    let number = 4 as usize;
    let trait_map = {
        let mut map = HashMap::new();
        map.insert("RelayerFactory".to_owned(), 1);
        map.insert("RelayerMachine".to_owned(), 2);
        map.insert("HelloFactory".to_owned(), 3);
        map.insert("HelloRobot".to_owned(), 4);
        map.insert("Schedule".to_owned(), 5);
        map
    };

    let mut modules = Modules::new();

    for i in 0..number {
        let ctx = executor::execute::<I, E>(mod_relayer_path).unwrap();
        let name = format!("Module{}", i);
        let args = serde_cbor::to_vec(&(number, i)).unwrap();
        modules.insert(name.clone(), FmlModule::new(ctx, trait_map.clone(), name, args.clone()));
    }
    {
        let ctx = executor::execute::<I, E>(mod_scheduler_path).unwrap();
        let name = "Schedule".to_owned();
        let args = serde_cbor::to_vec(&(number, SERVER_THREADS)).unwrap();
        modules.insert(name.clone(), FmlModule::new(ctx, trait_map, name, args));
    }

    link_all(&modules);
    exchange(&modules);

    let mut joins = Vec::new();
    let barrier = Arc::new(Barrier::new(number));

    // we need to keep schedule module alive
    let mut schedule = None;
    for (key, module) in modules.drain() {
        if key == "Schedule" {
            assert!(schedule.replace(module).is_none());
            continue
        }
        let b = barrier.clone();
        joins.push(thread::spawn(move || {
            module.debug(Vec::new());
            b.wait();
        }));
    }

    while let Some(x) = joins.pop() {
        x.join().unwrap();
    }
}

use super::*;
use crate::key::{end_test, start_test};
use cbsb::execution::executor::{Executable, PlainThread};
use cbsb::ipc::intra::Intra;
use cbsb::ipc::DefaultIpc;

fn register() -> (String, String) {
    let name1 = cbsb::ipc::generate_random_name();
    let name2 = cbsb::ipc::generate_random_name();
    executor::add_function_pool(name1.clone(), Arc::new(crate::mod_relayer::main_like));
    executor::add_function_pool(name2.clone(), Arc::new(crate::mod_scheduler::main_like));
    (name1, name2)
}

#[test]
fn fml_test_relay1() {
    let (mod_relayer_path, mod_scheduler_path) = register();
    for _ in 0..4 {
        let k = start_test();
        run::<Intra, PlainThread>(&mod_relayer_path, &mod_scheduler_path);
        end_test(k);
    }
}

#[test]
fn fml_test_relay2() {
    let (mod_relayer_path, mod_scheduler_path) = register();
    for _ in 0..4 {
        let k = start_test();
        run::<Intra, PlainThread>(&mod_relayer_path, &mod_scheduler_path);
        end_test(k);
    }
}

#[test]
fn fml_test_relay3() {
    let (mod_relayer_path, mod_scheduler_path) = register();
    for _ in 0..4 {
        let k = start_test();
        run::<Intra, PlainThread>(&mod_relayer_path, &mod_scheduler_path);
        end_test(k);
    }
}

#[test]
fn fml_test_complex_relay1() {
    for _ in 0..4 {
        let k = start_test();
        run::<DefaultIpc, Executable>(
            "./../target/debug/test_mod_relayer_rs",
            "./../target/debug/test_mod_scheduler_rs",
        );
        end_test(k);
    }
}

#[test]
fn fml_test_complex_relay2() {
    for _ in 0..4 {
        let k = start_test();
        run::<DefaultIpc, Executable>(
            "./../target/debug/test_mod_relayer_rs",
            "./../target/debug/test_mod_scheduler_rs",
        );
        end_test(k);
    }
}
