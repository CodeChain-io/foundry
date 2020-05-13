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

pub fn run<I: Ipc + 'static + LinkMessage, E: Executor + 'static>(mod_path: &str, trial: usize, number: usize) {
    for _ in 0..trial {
        // If not there might be an inevitable deadlock
        assert!(number <= SERVER_THREADS);

        let args = serde_cbor::to_vec(&number).unwrap();
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
            let name = format!("Module{}", i);
            let ctx = executor::execute::<I, E>(mod_path).unwrap();
            modules.insert(name.clone(), FmlModule::new(ctx, trait_map.clone(), name, args.clone()));
        }

        link_all(&modules);
        exchange(&modules);

        let mut joins = Vec::new();
        let barrier = Arc::new(Barrier::new(number));

        for (_, module) in modules.drain() {
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
}

use super::*;
use crate::key::{end_test, start_test};
use cbsb::execution::executor::{Executable, PlainThread};
use cbsb::ipc::intra::Intra;
use cbsb::ipc::DefaultIpc;

fn register() -> String {
    let name = cbsb::ipc::generate_random_name();
    executor::add_function_pool(name.clone(), Arc::new(crate::mod_hello::main_like));
    name
}

// testing these with different parameters isn't actually meaningful by each, but
// this is for testing the parallelism of multiple unit tests. (They share the global variables)
#[test]
fn fml_test_hello1() {
    let name = register();
    for _ in 0..4 {
        let k = start_test();

        run::<Intra, PlainThread>(&name, 8, 6);
        end_test(k);
    }
}

#[test]
fn fml_test_hello2() {
    let name = register();
    for _ in 0..4 {
        let k = start_test();
        run::<Intra, PlainThread>(&name, 8, 5);
        end_test(k);
    }
}

#[test]
fn fml_test_hello3() {
    let name = register();
    for _ in 0..4 {
        let k = start_test();
        run::<Intra, PlainThread>(&name, 8, 4);
        end_test(k);
    }
}

#[test]
fn fml_test_hello4() {
    let name = register();
    for _ in 0..4 {
        let k = start_test();
        run::<Intra, PlainThread>(&name, 8, 3);
        end_test(k);
    }
}

#[test]
fn fml_test_hello_binary1() {
    for _ in 0..3 {
        let k = start_test();
        run::<DefaultIpc, Executable>("./../target/debug/test_mod_hello_rs", 8, 6);
        end_test(k);
    }
}

#[test]
fn fml_test_hello_binary2() {
    for _ in 0..3 {
        let k = start_test();
        run::<DefaultIpc, Executable>("./../target/debug/test_mod_hello_rs", 8, 4);
        end_test(k);
    }
}
