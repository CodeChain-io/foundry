// Copyright 2018, 2020 Kodebox, Inc.
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

use ccore::BlockChainClient;
use ccore::Client;

use ckey::{Ed25519KeyPair, Generator, KeyPairTrait, Random};
use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use codechain_timestamp::account::TxHello;
use codechain_timestamp::common::*;
use coordinator::Transaction;
use ctypes::BlockId;
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

fn tx_hello(public: &Public, private: &Private, seq: u64) -> Transaction {
    let tx = TxHello;
    let tx = UserTransaction {
        seq,
        network_id: Default::default(),
        action: tx,
    };
    let tx_hash = tx.hash();
    let tx = SignedTransaction {
        signature: ckey::sign(tx_hash.as_bytes(), private),
        signer_public: *public,
        tx,
    };
    Transaction::new("account".to_owned(), serde_cbor::to_vec(&tx).unwrap())
}

pub fn inject_hello_txes(client: Arc<Client>) {
    let mut last_block_num = client.block_number(&BlockId::Latest).unwrap();

    let total_block_num = 4;
    let tx_per_step = 20;

    let user1: Ed25519KeyPair = Random.generate().unwrap();
    let mut seq = 0;

    for _ in 0..total_block_num {
        let mut success = false;
        for _ in 0..1000 {
            sleep(Duration::from_millis(100));

            for s in 0..tx_per_step {
                let _ = client.queue_own_transaction(tx_hello(user1.public(), user1.private(), seq + s));
            }

            let current_block_num = client.block_number(&BlockId::Latest).unwrap();
            if current_block_num == last_block_num {
                continue
            }

            let mut count = 0;

            for block_num in (last_block_num + 1)..current_block_num {
                count += client.block_body(&BlockId::Number(block_num)).unwrap().transactions_count();
            }
            if count == tx_per_step as usize {
                last_block_num = current_block_num;
                success = true;
                break
            } else {
                continue
            }
        }
        assert!(success);
        seq += tx_per_step;
    }

    let session = client.new_session(BlockId::Latest);
    let result = client.graphql_handlers().get("module-account").unwrap().execute(
        session,
        &format!("{{ account(public: \"{}\") {{ seq }} }}", hex::encode(user1.public().as_ref())),
        "{}",
    );
    assert_eq!(result, r#"{"data":{"account":{"seq":80}}}"#);
    client.end_session(session);
}

fn test_query(public: &Public) -> (std::collections::HashMap<String, String>, String) {
    let public_str = hex::encode(public.as_ref());
    let graphql_query = format!("{{ account(public: \"{}\") {{ seq }} }}", public_str);
    let variables = "{}".to_owned();
    (
        (vec![("query".to_owned(), graphql_query), ("variables".to_owned(), variables)]).into_iter().collect(),
        r#"{"data":{"account":{"seq":1}}}"#.to_string(),
    )
}

pub fn graphql(client: Arc<Client>) {
    let mut rt = tokio::runtime::Runtime::new().unwrap();
    rt.block_on(async {
        let user1: Ed25519KeyPair = Random.generate().unwrap();
        client.queue_own_transaction(tx_hello(user1.public(), user1.private(), 0)).unwrap();
        sleep(Duration::from_millis(4000));

        let client = awc::Client::new();
        let (query, expected) = test_query(user1.public());
        let request = client.get(&format!("http://localhost:{}/module-account/graphql", 1234)).query(&query).unwrap();
        let response_bytes = request.send().await.unwrap().body().await.unwrap();
        let response = std::str::from_utf8(&response_bytes).expect("GraphQL server must return utf8-encoded string");
        assert_eq!(response, expected);
    });
}
