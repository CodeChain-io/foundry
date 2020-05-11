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
use cbsb::ipc::intra::Intra;
use cbsb::ipc::multiplex::{Forward, Multiplexer};
use cbsb::ipc::Ipc;
use cbsb::ipc::{IpcRecv, IpcSend, Terminate};
use std::sync::{Arc, Barrier};
use std::thread;

type IpcScheme = cbsb::ipc::servo_channel::ServoChannel;

// CI server is really slow for this. Usually 10 is ok.
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(10000);

fn simple_thread(args: Vec<String>) {
    let ctx = executee::start::<Intra>(args);
    let r = ctx.ipc.as_ref().unwrap().recv(Some(TIMEOUT)).unwrap();
    assert_eq!(r, b"Hello?\0");
    ctx.ipc.as_ref().unwrap().send(b"I'm here!\0");
    ctx.terminate();
}

fn simple_executor<I: Ipc + 'static, E: executor::Executor>(path: &str) {
    let ctx = executor::execute::<I, E>(path).unwrap();
    ctx.ipc.send(b"Hello?\0");
    let r = ctx.ipc.recv(Some(TIMEOUT)).unwrap();
    assert_eq!(r, b"I'm here!\0");
    ctx.terminate();
}

#[test]
fn execute_simple_rust() {
    simple_executor::<IpcScheme, executor::Executable>("./../target/debug/test_simple_rs");
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

    let t1 = thread::spawn(|| simple_executor::<IpcScheme, executor::Executable>("./../target/debug/test_simple_rs"));
    let t2 = thread::spawn(|| simple_executor::<IpcScheme, executor::Executable>("./../target/debug/test_simple_rs"));
    let t3 = thread::spawn(|| simple_executor::<IpcScheme, executor::Executable>("./../target/debug/test_simple_rs"));

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

#[test]
fn execute_simple_intra_massive() {
    let name = cbsb::ipc::generate_random_name();
    executor::add_function_pool(name.clone(), Arc::new(simple_thread));

    let mut threads = Vec::new();
    for _ in 0..32 {
        let name = name.clone();
        threads.push(thread::spawn(move || {
            let mut ctxs = Vec::new();
            for _ in 0..32 {
                ctxs.push(executor::execute::<Intra, executor::PlainThread>(&name).unwrap());
            }

            for ctx in &ctxs {
                ctx.ipc.send(b"Hello?\0");
            }

            for ctx in &ctxs {
                let r = ctx.ipc.recv(Some(TIMEOUT)).unwrap();
                assert_eq!(r, b"I'm here!\0");
            }

            while let Some(x) = ctxs.pop() {
                x.terminate();
            }
        }))
    }

    while let Some(x) = threads.pop() {
        x.join().unwrap();
    }
}

fn setup_ipc<I: Ipc + 'static>() -> (I, I) {
    let (c1, c2) = I::arguments_for_both_ends();
    let d1 = thread::spawn(|| I::new(c1));
    let d2 = I::new(c2);
    let d1 = d1.join().unwrap();
    (d1, d2)
}

#[test]
fn terminator_socket() {
    let (d1, d2) = setup_ipc::<IpcScheme>();
    let terminator = d1.create_terminator();
    let barrier = Arc::new(Barrier::new(2));
    let barrier_ = barrier.clone();
    let t = thread::spawn(move || {
        assert_eq!(d1.recv(None).unwrap(), vec![1, 2, 3]);
        barrier_.wait();
        assert_eq!(d1.recv(None).unwrap_err(), cbsb::ipc::RecvError::Termination)
    });
    d2.send(&[1, 2, 3]);
    barrier.wait();
    terminator.terminate();
    t.join().unwrap();
}

#[test]
fn terminator_intra() {
    let (d1, d2) = setup_ipc::<Intra>();
    let terminator = d1.create_terminator();
    let barrier = Arc::new(Barrier::new(2));
    let barrier_ = barrier.clone();
    let t = thread::spawn(move || {
        assert_eq!(d1.recv(None).unwrap(), vec![1, 2, 3]);
        barrier_.wait();
        assert_eq!(d1.recv(None).unwrap_err(), cbsb::ipc::RecvError::Termination)
    });
    d2.send(&[1, 2, 3]);
    barrier.wait();
    terminator.terminate();
    t.join().unwrap();
}

/// DomainSocket send must be a blocking operation; if not then this bursting traffic
/// will make the send() cause the "Resource temporarily unavailable" error.
#[test]
fn socket_must_block() {
    let n = 200;
    let packet_size = 3000;
    let (d1, d2) = setup_ipc::<IpcScheme>();
    let terminator = d1.create_terminator();
    let barrier = Arc::new(Barrier::new(2));
    let barrier_ = barrier.clone();
    let t = thread::spawn(move || {
        for i in 0..n {
            let r = d1.recv(None).unwrap();
            assert!(r.iter().all(|&x| x == (i % 256) as u8));
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        barrier_.wait();
        assert_eq!(d1.recv(None).unwrap_err(), cbsb::ipc::RecvError::Termination);
    });
    for i in 0..n {
        let mut data = Vec::<u8>::with_capacity(packet_size);
        for _ in 0..packet_size {
            data.push((i % 256) as u8);
        }
        d2.send(&data);
    }
    barrier.wait();
    terminator.terminate();
    t.join().unwrap();
}

struct TestForward;

impl Forward for TestForward {
    fn forward(data: &[u8]) -> usize {
        if data[0] == b'0' {
            0
        } else {
            1
        }
    }
}

#[test]
fn multiplexer() {
    let (c1, c2) = IpcScheme::arguments_for_both_ends();
    let d1 = thread::spawn(|| IpcScheme::new(c1));
    let d2 = IpcScheme::new(c2);
    let d1 = d1.join().unwrap();

    let (s1, r1) = d1.split();
    let (s2, r2) = d2.split();

    let (mut multiplexed1, multiplxer1) = Multiplexer::create::<TestForward, _, _>(s1, r1, 2, 100);
    let (mut multiplexed2, multiplxer2) = Multiplexer::create::<TestForward, _, _>(s2, r2, 2, 100);

    // multiplxed channel 1
    let (s1_1, r1_1) = multiplexed1.pop().unwrap();
    let (s2_1, r2_1) = multiplexed2.pop().unwrap();

    // multiplxed channel 0
    let (s1_2, r1_2) = multiplexed1.pop().unwrap();
    let (s2_2, r2_2) = multiplexed2.pop().unwrap();

    s1_1.send(b"11".to_vec()).unwrap();
    assert_eq!(r2_1.recv().unwrap(), b"11");

    s2_1.send(b"12".to_vec()).unwrap();
    assert_eq!(r1_1.recv().unwrap(), b"12");

    s2_2.send(b"03".to_vec()).unwrap();
    assert_eq!(r1_2.recv().unwrap(), b"03");

    s1_2.send(b"04".to_vec()).unwrap();
    assert_eq!(r2_2.recv().unwrap(), b"04");

    // we have to drop the multiplexer itself first
    drop(multiplxer1);
    drop(multiplxer2);
}
