// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

// You should run `cargo test` with `--test-threads=1`, if you use `integration-test` feature.
#![cfg(feature = "integration-test")]

extern crate foundry_integration_test as test_common;

use ckey::{Ed25519KeyPair, Generator, KeyPairTrait, Random};
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::Value;
use std::time::Duration;
use test_common::*;
use tokio::time::delay_for;

fn run_node_override(arg: FoundryArgs) -> FoundryNode {
    run_node(
        RunNodeArgs {
            foundry_path: "../target/debug/foundry".to_string(),
            rust_log: "warn".to_string(),
            app_desc_path: "../timestamp/app-desc.toml".to_string(),
            link_desc_path: "../timestamp/link-desc.toml".to_string(),
            config_path: "config.tendermint-solo.ini".to_string(),
        },
        arg,
    )
}

const GRAPHQL_PORT: u16 = 4444;

fn simple_solo_node() -> FoundryArgs {
    FoundryArgs {
        graphql_port: GRAPHQL_PORT,
        mem_pool_size: 32768,
        engine_signer: "rjmxg19kCmkCxROEoV0QYsrDpOYsjQwusCtN5_oKMEzk-I6kgtAtc0".to_owned(),
        port: 3333,
        bootstrap_addresses: Vec::new(),
        password_path: "./password.json".to_owned(),
    }
}

#[actix_rt::test]
async fn run() {
    let _node = run_node_override(simple_solo_node());
    delay_for(Duration::from_secs(3)).await;
}

#[actix_rt::test]
async fn ping() {
    let _node = run_node_override(simple_solo_node());
    delay_for(Duration::from_secs(3)).await;
    let x = request_query(GRAPHQL_PORT, "ping", "aaaa", "aaaa").await;
    assert_eq!(x, "Module not found: ping");
}

#[actix_rt::test]
async fn track_blocks() {
    let _node = run_node_override(simple_solo_node());
    delay_for(Duration::from_secs(3)).await;

    let start_block = get_latest_block(GRAPHQL_PORT).await;
    while get_latest_block(GRAPHQL_PORT).await < start_block + 15 {
        delay_for(Duration::from_secs(1)).await;
    }
}

#[actix_rt::test]
async fn sequence_management1() {
    let _node = run_node_override(simple_solo_node());
    delay_for(Duration::from_secs(3)).await;

    let user: Ed25519KeyPair = Random.generate().unwrap();

    // valid
    let tx = create_tx_hello(GRAPHQL_PORT, user.public(), user.private(), 0).await;
    send_tx(GRAPHQL_PORT, tx.tx_type(), tx.body()).await.unwrap();

    // invalid
    let tx = create_tx_hello(GRAPHQL_PORT, user.public(), user.private(), 100).await;
    send_tx(GRAPHQL_PORT, tx.tx_type(), tx.body()).await.unwrap();

    delay_for(Duration::from_secs(6)).await;

    let latest = get_latest_block(GRAPHQL_PORT).await;
    let mut num = 0;
    let query = "query Test($number: Int!) {
        block(number: $number) {
            transactions { txType }
        }
    }";
    for i in 0..latest {
        let query_result = request_query(GRAPHQL_PORT, "engine", query, &format!(r#"{{"number": {}}}"#, i)).await;
        let value: Value = serde_json::from_str(&query_result).unwrap();
        let txes: Vec<Value> = serde_json::from_value(value["data"]["block"]["transactions"].clone()).unwrap();
        num += txes.len();
    }
    assert_eq!(num, 1);

    let query = format!("{{ account(public: \"{}\") {{ seq }} }}", hex::encode(user.public().as_ref()));
    let query_result = request_query(GRAPHQL_PORT, "module-account", &query, "{}").await;
    let value: Value = serde_json::from_str(&query_result).unwrap();
    assert_eq!(value["data"]["account"]["seq"], 1);
}

#[actix_rt::test]
async fn sequence_management2() {
    let _node = run_node_override(simple_solo_node());
    delay_for(Duration::from_secs(3)).await;

    let user: Ed25519KeyPair = Random.generate().unwrap();
    let tx_num_per_step = 100;
    let mut rng = thread_rng();

    for i in 0..4 {
        let mut txes = Vec::new();
        for j in 0..tx_num_per_step {
            txes.push(create_tx_hello(GRAPHQL_PORT, user.public(), user.private(), i * tx_num_per_step + j).await);
            // Far future tx
            txes.push(create_tx_hello(GRAPHQL_PORT, user.public(), user.private(), 100000 + j).await);
            // invalid tx
            if i > 0 {
                txes.push(
                    create_tx_hello(GRAPHQL_PORT, user.public(), user.private(), (i - 1) * tx_num_per_step + j).await,
                );
            }
        }
        txes.shuffle(&mut rng);

        for tx in txes {
            let _ = send_tx(GRAPHQL_PORT, tx.tx_type(), tx.body()).await;
        }

        delay_for(Duration::from_secs(8)).await;

        let query = format!("{{ account(public: \"{}\") {{ seq }} }}", hex::encode(user.public().as_ref()));
        let query_result = request_query(GRAPHQL_PORT, "module-account", &query, "{}").await;
        let value: Value = serde_json::from_str(&query_result).unwrap();
        assert_eq!(value["data"]["account"]["seq"], tx_num_per_step * (i + 1));
    }
}

#[actix_rt::test]
async fn events() {
    let _node = run_node_override(simple_solo_node());
    delay_for(Duration::from_secs(3)).await;

    let user: Ed25519KeyPair = Random.generate().unwrap();

    let tx1 = create_tx_hello(GRAPHQL_PORT, user.public(), user.private(), 0).await;
    send_tx(GRAPHQL_PORT, tx1.tx_type(), tx1.body()).await.unwrap();

    // invalid
    let tx2 = create_tx_hello(GRAPHQL_PORT, user.public(), user.private(), 100).await;
    send_tx(GRAPHQL_PORT, tx2.tx_type(), tx2.body()).await.unwrap();

    delay_for(Duration::from_secs(4)).await;

    let result = get_event(GRAPHQL_PORT, *tx1.hash()).await;

    assert_eq!(1, result.len());
    assert_eq!(*tx1.body(), result[0]);

    let result = get_event(GRAPHQL_PORT, *tx2.hash()).await;
    assert!(result.is_empty());

    assert!(get_tx(GRAPHQL_PORT, *tx1.hash()).await.unwrap() <= get_latest_block(GRAPHQL_PORT).await);
}
