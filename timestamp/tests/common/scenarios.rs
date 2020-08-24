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

use super::mock_coordinator::Context;
use ccrypto::blake256;
use ckey::{Ed25519KeyPair, Generator, KeyPairTrait, Random};
use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::context::SubStorageAccess;
use coordinator::{Transaction, TransactionWithMetadata, TxOrigin};
use parking_lot::RwLock;
use primitives::H256;
use rand::prelude::*;
use rand::seq::IteratorRandom;
use remote_trait_object::ServiceRef;
use std::collections::HashMap;
use std::sync::Arc;
use timestamp::common::*;

fn setup_stateful(ctx: &RwLock<Context>) {
    let names: Vec<String> = ctx.write().statefuls.keys().cloned().collect();
    let mut storages: HashMap<String, Arc<RwLock<dyn SubStorageAccess>>> =
        names.iter().map(|name| (name.to_owned(), ctx.write().get_storage(name.to_owned()).to_owned())).collect();

    for (name, stateful) in ctx.write().statefuls.iter_mut() {
        stateful.set_storage(ServiceRef::create_export(storages.remove(name).unwrap()))
    }
}

fn tx_hello(public: &Public, private: &Private, seq: u64) -> Transaction {
    let tx = timestamp::account::TxHello;
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

fn tx_stamp(public: &Public, private: &Private, seq: u64, contents: &str) -> Transaction {
    let tx = timestamp::stamp::TxStamp {
        hash: blake256(contents),
    };
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
    Transaction::new("stamp".to_owned(), serde_cbor::to_vec(&tx).unwrap())
}

fn tx_token_transfer(public: &Public, private: &Private, seq: u64, receiver: Public, issuer: H256) -> Transaction {
    let tx = timestamp::token::Action::TransferToken(timestamp::token::ActionTransferToken {
        issuer,
        receiver,
    });
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
    Transaction::new("token".to_owned(), serde_cbor::to_vec(&tx).unwrap())
}

pub fn simple1(ctx: &RwLock<Context>) {
    setup_stateful(ctx);

    let user1: Ed25519KeyPair = Random.generate().unwrap();
    let user2: Ed25519KeyPair = Random.generate().unwrap();

    let mut stampers = HashMap::new();
    stampers.insert(user1.public(), 1usize);
    stampers.insert(user2.public(), 0usize);

    ctx.write().init_genesises.get_mut("stamp").unwrap().init_genesis(&serde_cbor::to_vec(&stampers).unwrap());

    let stamp_by_user1 = tx_stamp(user1.public(), user1.private(), 0, "Hello");
    let stamp_by_user2 = tx_stamp(user2.public(), user2.private(), 0, "Hello");

    ctx.write().tx_owners.get_mut("stamp").unwrap().execute_transaction(&stamp_by_user1).unwrap();
    assert!(ctx.write().tx_owners.get_mut("stamp").unwrap().execute_transaction(&stamp_by_user2).is_err());
}

pub fn multiple(ctx: &RwLock<Context>) {
    let mut rng = rand::thread_rng();
    let stamp_issuer = blake256("stamp");

    setup_stateful(ctx);

    let n = 32;
    let mut users: Vec<(Ed25519KeyPair, u64)> = (0..n).map(|_| (Random.generate().unwrap(), 0)).collect();
    let mut tokens: Vec<usize> = (0..n).choose_multiple(&mut rng, n / 2).into_iter().collect();

    let mut stampers = HashMap::new();
    for token_owner in tokens.iter() {
        stampers.insert(users[*token_owner].0.public(), 1usize);
    }
    ctx.write().init_genesises.get_mut("stamp").unwrap().init_genesis(&serde_cbor::to_vec(&stampers).unwrap());

    for _ in 0..100 {
        let m = rng.gen_range(1, n);
        let stampers = (0..n).choose_multiple(&mut rng, m);
        for i in stampers {
            let (key, seq) = &mut users[i];
            let tx = tx_stamp(key.public(), key.private(), *seq, "Hello");

            if tokens.iter().any(|&x| x == i) {
                ctx.write().tx_owners.get_mut("stamp").unwrap().execute_transaction(&tx).unwrap();
                *seq += 1;
            } else {
                assert!(ctx.write().tx_owners.get_mut("stamp").unwrap().execute_transaction(&tx).is_err());
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
                ctx.write().tx_owners.get_mut("token").unwrap().execute_transaction(&tx).unwrap();
                *seq += 1;
                *owner = receiver;
            } else {
                assert!(ctx.write().tx_owners.get_mut("token").unwrap().execute_transaction(&tx).is_err());
            }
        }
    }
}

pub fn sort(ctx: &RwLock<Context>) {
    setup_stateful(ctx);

    let user_num = 10;
    let mut rng = rand::thread_rng();
    let users: Vec<Ed25519KeyPair> = (0..user_num).map(|_| Random.generate().unwrap()).collect();

    let mut txes: Vec<Transaction> = Vec::new();
    for user in users {
        let n = rng.gen_range(10, 100);
        for i in 0..n {
            let tx = tx_hello(user.public(), user.private(), i);
            txes.push(tx);
        }

        // invalid transactions
        for i in (n + 100)..(n + 200) {
            let tx = tx_hello(user.public(), user.private(), i);
            txes.push(tx);
        }
    }
    txes.shuffle(&mut rng);
    let txes: Vec<TransactionWithMetadata> = txes
        .into_iter()
        .map(|tx| TransactionWithMetadata {
            tx,
            origin: TxOrigin::Local,
            inserted_block_number: 1,
            inserted_timestamp: 1234,
            insertion_id: 1234,
        })
        .collect();

    let result = ctx.read().tx_sorter.as_ref().unwrap().sort_txs(&txes);
    assert_eq!(result.invalid.len(), 100 * user_num);

    // all transactions must succeed
    for tx in result.sorted {
        ctx.write().tx_owners.get_mut("account").unwrap().execute_transaction(&txes[tx].tx).unwrap();
    }
}

pub fn query(ctx: &RwLock<Context>) {
    setup_stateful(ctx);

    let user: Ed25519KeyPair = Random.generate().unwrap();

    let n = 21;
    for i in 0..n {
        let tx = tx_hello(user.public(), user.private(), i);
        ctx.write().tx_owners.get_mut("account").unwrap().execute_transaction(&tx).unwrap();
    }

    let public_str = hex::encode(user.public().as_ref());
    let result =
        ctx.read().handle_graphqls.get("account").unwrap().execute(&format!(
            "{{ withBlockHeight(height: null) {{ account(public: \"{}\") {{ seq }} }} }}",
            public_str
        ));
    assert_eq!(r#"{"data":{"withBlockHeight":{"account":{"seq":21}}}}"#, result);
}
