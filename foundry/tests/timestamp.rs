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
use ccrypto::blake256;
use ckey::{Ed25519KeyPair, Generator, KeyPairTrait, Random};
use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use codechain_timestamp::account::TxHello;
use codechain_timestamp::common::*;
use coordinator::context::SubStorageAccess;
use coordinator::{Transaction, TransactionWithMetadata, TxOrigin};
use ctypes::BlockId;
use parking_lot::RwLock;
use primitives::H256;
use rand::prelude::*;
use rand::seq::IteratorRandom;
use remote_trait_object::{Service, ServiceRef};
use std::collections::HashMap;
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

    let total_block_num = 8;
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
}
