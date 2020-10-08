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

use ccrypto::blake256;
use ckey::{Ed25519KeyPair, Generator, KeyPairTrait, Random};
use common::*;
use coordinator::module::SessionId;
use coordinator::{AppDesc, Coordinator};
use rand::prelude::*;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;

mod timestamp_setup {
    use super::*;
    use codechain_module::impls::process::{ExecutionScheme, SingleProcess};
    use codechain_module::MODULE_INITS;
    use foundry_module_rt::start;
    use foundry_process_sandbox::execution::executor::add_function_pool;
    use linkme::distributed_slice;
    use std::sync::Arc;

    #[distributed_slice(MODULE_INITS)]
    fn account() {
        static VISIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        if VISIT.compare_and_swap(true, false, std::sync::atomic::Ordering::SeqCst) {
            add_function_pool(
                "a010000000012345678901234567890123456789012345678901234567890123".to_owned(),
                Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::account::Module>),
            );
        }
    }

    #[distributed_slice(MODULE_INITS)]
    fn staking() {
        static VISIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        if VISIT.compare_and_swap(true, false, std::sync::atomic::Ordering::SeqCst) {
            add_function_pool(
                "a020000000012345678901234567890123456789012345678901234567890123".to_owned(),
                Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::staking::Module>),
            );
        }
    }

    #[distributed_slice(MODULE_INITS)]
    fn stamp() {
        static VISIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        if VISIT.compare_and_swap(true, false, std::sync::atomic::Ordering::SeqCst) {
            add_function_pool(
                "a030000000012345678901234567890123456789012345678901234567890123".to_owned(),
                Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::stamp::Module>),
            );
        }
    }

    #[distributed_slice(MODULE_INITS)]
    fn token() {
        static VISIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        if VISIT.compare_and_swap(true, false, std::sync::atomic::Ordering::SeqCst) {
            add_function_pool(
                "a040000000012345678901234567890123456789012345678901234567890123".to_owned(),
                Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::token::Module>),
            );
        }
    }

    #[distributed_slice(MODULE_INITS)]
    fn sorting() {
        static VISIT: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(true);
        if VISIT.compare_and_swap(true, false, std::sync::atomic::Ordering::SeqCst) {
            add_function_pool(
                "a050000000012345678901234567890123456789012345678901234567890123".to_owned(),
                Arc::new(start::<<SingleProcess as ExecutionScheme>::Ipc, timestamp::sorting::Module>),
            );
        }
    }
}

fn app_desc_path() -> &'static str {
    if std::path::Path::exists(std::path::Path::new("../app-desc.yml")) {
        "../app-desc.yml"
    } else {
        "./app-desc.yml"
    }
}

fn app_desc() -> AppDesc {
    let app_desc = std::fs::read_to_string(app_desc_path()).unwrap();
    let mut app_desc = AppDesc::from_str(&app_desc).unwrap();
    // TODO: proper parameter merging must be implemented with actual parameters from configs
    app_desc.merge_params(&std::collections::BTreeMap::new()).unwrap();
    #[cfg(feature = "multi-process")]
    {
        app_desc.default_sandboxer = "multi-process".to_owned();
    }
    app_desc
}

#[test]
fn weave() {
    let c = Coordinator::from_app_desc(&app_desc()).unwrap();

    assert_eq!(c.services().stateful.lock().len(), 2);
    assert_eq!(c.services().init_genesis.len(), 2);
    assert_eq!(c.services().tx_owner.len(), 3);
    assert_eq!(c.services().handle_graphqls.len(), 2);
}

#[test]
fn weave_conccurent() {
    for i in 0..8 {
        let n = 8;
        let mut joins = Vec::new();
        for _ in 0..n {
            joins.push(std::thread::spawn(|| {
                let c = Coordinator::from_app_desc(&app_desc()).unwrap();

                assert_eq!(c.services().stateful.lock().len(), 2);
                assert_eq!(c.services().init_genesis.len(), 2);
                assert_eq!(c.services().tx_owner.len(), 3);
                assert_eq!(c.services().handle_graphqls.len(), 2);
            }))
        }
        for j in joins {
            j.join().unwrap();
        }
        println!("{}", i);
    }
}

#[test]
fn simple1() {
    let coordinator = Coordinator::from_app_desc(&app_desc()).unwrap();
    set_empty_session(0, &coordinator);
    let services = Services::new(&coordinator);

    let user1: Ed25519KeyPair = Random.generate().unwrap();
    let user2: Ed25519KeyPair = Random.generate().unwrap();

    let mut stampers = HashMap::new();
    stampers.insert(user1.public(), 1usize);
    stampers.insert(user2.public(), 0usize);

    services.init_genesis.get("module-stamp").unwrap().init_genesis(0, &serde_cbor::to_vec(&stampers).unwrap());

    let stamp_by_user1 = tx_stamp(user1.public(), user1.private(), 0, "Hello");
    let stamp_by_user2 = tx_stamp(user2.public(), user2.private(), 0, "Hello");

    services.tx_owner.get("stamp").unwrap().execute_transaction(0, &stamp_by_user1).unwrap();
    assert!(services.tx_owner.get("stamp").unwrap().execute_transaction(0, &stamp_by_user2).is_err());
}

fn run_massive_token_exchange(id: SessionId, c: &Coordinator) {
    set_empty_session(id, &c);
    let services = Services::new(&c);

    let mut rng = rand::thread_rng();
    let stamp_issuer = blake256("stamp");

    let n = 16;
    let mut users: Vec<(Ed25519KeyPair, u64)> = (0..n).map(|_| (Random.generate().unwrap(), 0)).collect();
    let mut tokens: Vec<usize> = (0..n).choose_multiple(&mut rng, n / 2).into_iter().collect();

    let mut stampers = HashMap::new();
    for token_owner in tokens.iter() {
        stampers.insert(users[*token_owner].0.public(), 1usize);
    }
    services.init_genesis.get("module-stamp").unwrap().init_genesis(id, &serde_cbor::to_vec(&stampers).unwrap());

    for _ in 0..100 {
        let m = rng.gen_range(1, n);
        let stampers = (0..n).choose_multiple(&mut rng, m);
        for i in stampers {
            let (key, seq) = &mut users[i];
            let tx = tx_stamp(key.public(), key.private(), *seq, "Hello");

            if tokens.iter().any(|&x| x == i) {
                services.tx_owner.get("stamp").unwrap().execute_transaction(id, &tx).unwrap();
                *seq += 1;
            } else {
                assert!(services.tx_owner.get("stamp").unwrap().execute_transaction(id, &tx).is_err());
            }
        }

        let m = rng.gen_range(1, n);
        let transferers = (0..n).choose_multiple(&mut rng, m);
        for i in transferers {
            let receiver = rng.gen_range(0, n);
            let receiver_key = *users[receiver].0.public();
            let (key, seq) = &mut users[i];
            let tx = tx_token_transfer(key.public(), key.private(), *seq, receiver_key, stamp_issuer);

            if receiver == i {
                continue
            }

            if let Some(owner) = tokens.iter_mut().find(|x| **x == i) {
                services.tx_owner.get("token").unwrap().execute_transaction(id, &tx).unwrap();
                *seq += 1;
                *owner = receiver;
            } else {
                assert!(services.tx_owner.get("token").unwrap().execute_transaction(id, &tx).is_err());
            }
        }
    }
}

#[test]
fn multiple() {
    let coordinator = Coordinator::from_app_desc(&app_desc()).unwrap();
    run_massive_token_exchange(0, &coordinator);
}

#[test]
fn multiple_concurrent() {
    let coordinator = Arc::new(Coordinator::from_app_desc(&app_desc()).unwrap());
    let mut joins = Vec::new();
    for i in 0..4 {
        let c = Arc::clone(&coordinator);
        joins.push(std::thread::spawn(move || run_massive_token_exchange(i, c.as_ref())));
    }
    for j in joins {
        j.join().unwrap();
    }
}

#[test]
fn query() {
    let coordinator = Coordinator::from_app_desc(&app_desc()).unwrap();
    set_empty_session(0, &coordinator);
    let services = Services::new(&coordinator);

    let user: Ed25519KeyPair = Random.generate().unwrap();

    let n = 21;
    for i in 0..n {
        let tx = tx_hello(user.public(), user.private(), i);
        services.tx_owner.get("account").unwrap().execute_transaction(0, &tx).unwrap();
    }

    let public_str = hex::encode(user.public().as_ref());
    let result = services.handle_graphqls.get("module-account").unwrap().execute(
        0,
        &format!("{{ account(public: \"{}\") {{ seq }} }}", public_str),
        "{}",
    );
    assert_eq!(r#"{"data":{"account":{"seq":21}}}"#, result);

    let result = services.handle_graphqls.get("module-account").unwrap().execute(
        0,
        include_str!("./common/query.graphql"),
        &format!("{{\"public\": \"{}\"}}", public_str),
    );
    assert_eq!(r#"{"data":{"account":{"seq":21}}}"#, result);
}

#[test]
fn query_tx() {
    let coordinator = Coordinator::from_app_desc(&app_desc()).unwrap();
    set_empty_session(0, &coordinator);
    let services = Services::new(&coordinator);

    let result =
        services.handle_graphqls.get("module-account").unwrap().execute(0, &format!("{{ txHello(seq: {}) }}", 0), "{}");

    let value: Value = serde_json::from_str(&result).unwrap();
    let tx = hex::decode(value["data"]["txHello"].as_str().unwrap()).unwrap();

    let user: Ed25519KeyPair = Random.generate().unwrap();
    let tx = sign_tx(user.public(), user.private(), "account".to_owned(), tx);

    services.tx_owner.get("account").unwrap().execute_transaction(0, &tx).unwrap();
}
