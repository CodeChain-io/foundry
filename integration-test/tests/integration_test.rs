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
use serde_json::Value;
use std::time::Duration;
use test_common::*;
use tokio::time::delay_for;

#[actix_rt::test]
async fn run() {
    let _node = run_node(4444);
    delay_for(Duration::from_secs(3)).await;
}

#[actix_rt::test]
async fn ping() {
    let _node = run_node(5555);
    delay_for(Duration::from_secs(3)).await;
    let x = request_query(5555, "ping", "aaaa", "aaaa").await;
    assert_eq!(x, "Module not found: ping");
}

#[actix_rt::test]
async fn track_blocks() {
    let port = 5555;
    let _node = run_node(port);
    delay_for(Duration::from_secs(3)).await;

    let start_block = get_latest_block(port).await;
    while get_latest_block(port).await < start_block + 15 {
        delay_for(Duration::from_secs(1)).await;
    }
}

#[actix_rt::test]
async fn send_hello_tx() {
    let port = 5555;
    let _node = run_node(port);
    delay_for(Duration::from_secs(3)).await;

    let user: Ed25519KeyPair = Random.generate().unwrap();

    // valid
    let tx = create_tx_hello(port, user.public(), user.private(), 0).await;
    send_tx(port, tx.tx_type(), tx.body()).await;

    // invalid
    let tx = create_tx_hello(port, user.public(), user.private(), 100).await;
    send_tx(port, tx.tx_type(), tx.body()).await;

    delay_for(Duration::from_secs(6)).await;

    let latest = get_latest_block(port).await;
    let mut num = 0;
    let query = "query Test($number: Int!) {
        block(number: $number) {
            transactions { txType }
        }
    }";
    for i in 0..latest {
        let query_result = request_query(port, "engine", query, &format!(r#"{{"number": {}}}"#, i)).await;
        let value: Value = serde_json::from_str(&query_result).unwrap();
        let txes: Vec<Value> = serde_json::from_value(value["data"]["block"]["transactions"].clone()).unwrap();
        num += txes.len();
    }
    assert_eq!(num, 1);
}
