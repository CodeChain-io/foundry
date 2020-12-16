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
use std::time::Duration;
use test_common::*;
use tokio::time::delay_for;

fn run_node_override(arg: FoundryArgs) -> FoundryNode {
    run_node(
        RunNodeArgs {
            foundry_path: "../target/debug/foundry".to_string(),
            rust_log: "warn".to_string(),
            app_desc_path: "../demo/app-desc.toml".to_string(),
            link_desc_path: "../demo/link-desc.toml".to_string(),
            config_path: "config.tendermint-solo.ini".to_string(),
        },
        arg,
    )
}

const GRAPHQL_PORT_BASE: u16 = 5000;

fn simple_multi_node(index: usize) -> FoundryArgs {
    let signers = [
        "rjmxg19kCmkCxROEoV0QYsrDpOYsjQwusCtN5_oKMEzk-I6kgtAtc0",
        "szff1322BHP3gsOuwFPDf-K8zvqSmNz4rj3CJirlQKFKWA_3c-Ytc0",
        "qwfj0xwkJQLV5iEGeaGeRfPA-TJX56Mnuq9fQD9coasmhanhck4tc0",
        "dbqtds3w6QnzEf0RXuQS7c_N6IzFBzcBAfdjWme5y0U5DxzLS14tc0",
    ];

    let mut bootstrap_addresses = Vec::new();
    for i in 0..4 {
        if i == index {
            continue
        }
        bootstrap_addresses.push(format!("127.0.0.1:{}", 3000 + i as u16));
    }

    FoundryArgs {
        graphql_port: GRAPHQL_PORT_BASE + index as u16,
        mem_pool_size: 32768,
        engine_signer: signers[index].to_owned(),
        port: 3000 + index as u16,
        bootstrap_addresses,
        password_path: "../demo/password.json".to_owned(),
    }
}

#[actix_rt::test]
async fn ping() {
    let _node: Vec<FoundryNode> = (0..4).into_iter().map(|i| run_node_override(simple_multi_node(i))).collect();
    delay_for(Duration::from_secs(3)).await;

    for i in 0..4 {
        let x = request_query(GRAPHQL_PORT_BASE + i, "ping", "aaaa", "aaaa").await;
        assert_eq!(x, "Module not found: ping");
    }
}

#[actix_rt::test]
async fn track_blocks() {
    let _node: Vec<FoundryNode> = (0..4).into_iter().map(|i| run_node_override(simple_multi_node(i))).collect();
    delay_for(Duration::from_secs(3)).await;

    let start_block = get_latest_block(GRAPHQL_PORT_BASE).await;
    while get_latest_block(GRAPHQL_PORT_BASE).await < start_block + 8 {
        delay_for(Duration::from_secs(1)).await;
    }

    for _ in 0..60 {
        delay_for(Duration::from_secs(1)).await;
        let mut success = true;
        for i in 0..4 {
            success = success && get_latest_block(GRAPHQL_PORT_BASE + i).await >= start_block + 8;
        }
        if success {
            return
        }
    }
    panic!("Failed to sync 4 nodes")
}

#[actix_rt::test]
async fn events_stored_in_every_nodes() {
    let _node: Vec<FoundryNode> = (0..4).into_iter().map(|i| run_node_override(simple_multi_node(i))).collect();
    delay_for(Duration::from_secs(3)).await;

    let user: Ed25519KeyPair = Random.generate().unwrap();

    let tx = create_tx_hello(GRAPHQL_PORT_BASE, user.public(), user.private(), 0).await;
    send_tx(GRAPHQL_PORT_BASE, tx.tx_type(), tx.body()).await.unwrap();

    delay_for(Duration::from_secs(4)).await;

    for i in 0..4 {
        let result = get_event(GRAPHQL_PORT_BASE + i, *tx.hash()).await;
        assert_eq!(1, result.len());
    }
}
