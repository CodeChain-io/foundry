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

extern crate codechain_basesandbox as cbsb;

use cbsb::execution::executee;
use cbsb::execution::executor;
use cbsb::ipc::domain_socket::DomainSocket;
use cbsb::ipc::intra::Intra;
use cbsb::ipc::Ipc;
use cbsb::ipc::{IpcRecv, IpcSend};
use std::sync::Arc;
use std::thread;

// CI server is really slow for this. Usually 10 is ok.
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10000);

fn simple_thread(args: Vec<String>) {
    let ctx = executee::start::<Intra>(args);
    let r = ctx.ipc.as_ref().unwrap().recv(Some(TIMEOUT)).unwrap();
    assert_eq!(r, b"Hello?\0");
    ctx.ipc.as_ref().unwrap().send(b"I'm here!\0");
    ctx.terminate();
}

fn simple_executor<I: Ipc, E: executor::Executor>(path: &str) {
    let ctx = executor::execute::<I, E>(path).unwrap();
    ctx.ipc.send(b"Hello?\0");
    let r = ctx.ipc.recv(Some(TIMEOUT)).unwrap();
    assert_eq!(r, b"I'm here!\0");
    ctx.terminate();
}

#[test]
fn execute_simple_rust() {
    simple_executor::<DomainSocket, executor::Executable>("./../target/debug/test_simple_rs");
}

#[test]
fn execute_simple_intra() {
    // Note that cargo unit tests might share global static variable.
    // You must use unique name per execution
    let name = cbsb::ipc::generate_random_name();
    executor::add_function_pool(name.clone(), Arc::new(simple_thread));
    simple_executor::<Intra, executor::PlainThread>(&name);
}

#[test]
fn execute_simple_multiple() {
    let name_source = cbsb::ipc::generate_random_name();
    executor::add_function_pool(name_source.clone(), Arc::new(simple_thread));

    let t1 =
        thread::spawn(|| simple_executor::<DomainSocket, executor::Executable>("./../target/debug/test_simple_rs"));
    let t2 =
        thread::spawn(|| simple_executor::<DomainSocket, executor::Executable>("./../target/debug/test_simple_rs"));
    let t3 =
        thread::spawn(|| simple_executor::<DomainSocket, executor::Executable>("./../target/debug/test_simple_rs"));

    let name = name_source.clone();
    let t4 = thread::spawn(move || simple_executor::<Intra, executor::PlainThread>(&name));
    let name = name_source.clone();
    let t5 = thread::spawn(move || simple_executor::<Intra, executor::PlainThread>(&name));
    let name = name_source;
    let t6 = thread::spawn(move || simple_executor::<Intra, executor::PlainThread>(&name));

    t1.join().unwrap();
    t2.join().unwrap();
    t3.join().unwrap();
    t4.join().unwrap();
    t5.join().unwrap();
    t6.join().unwrap();
}

#[test]
fn execute_simple_intra_complicated() {
    let name = cbsb::ipc::generate_random_name();
    executor::add_function_pool(name.clone(), Arc::new(simple_thread));
    let ctx1 = executor::execute::<Intra, executor::PlainThread>(&name).unwrap();
    let ctx2 = executor::execute::<Intra, executor::PlainThread>(&name).unwrap();

    ctx2.ipc.send(b"Hello?\0");
    ctx1.ipc.send(b"Hello?\0");

    let r = ctx1.ipc.recv(Some(TIMEOUT)).unwrap();
    assert_eq!(r, b"I'm here!\0");
    let r = ctx2.ipc.recv(Some(TIMEOUT)).unwrap();
    assert_eq!(r, b"I'm here!\0");

    ctx1.terminate();
    ctx2.terminate();
}
