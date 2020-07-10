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
use coordinator::types::Transaction;
use parking_lot::RwLock;
use primitives::H256;
use remote_trait_object::{Service, ServiceRef};
use std::collections::HashMap;
use timestamp::common::*;

#[derive(Default)]
pub struct MockDb {
    map: HashMap<H256, Vec<u8>>,
}

impl Service for MockDb {}

impl SubStorageAccess for MockDb {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.map.get(&blake256(key)).cloned()
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        self.map.insert(blake256(key), value);
    }

    fn remove(&mut self, key: &[u8]) {
        self.map.remove(&blake256(key));
    }

    fn has(&self, key: &[u8]) -> bool {
        self.map.get(&blake256(key)).is_some()
    }

    fn create_checkpoint(&mut self) {
        unimplemented!()
    }

    fn discard_checkpoint(&mut self) {
        unimplemented!()
    }

    fn revert_to_the_checkpoint(&mut self) {
        unimplemented!()
    }
}

fn tx_stamp(public: &Public, private: &Private, contents: &str) -> Transaction {
    let tx = timestamp::stamp::TxStamp {
        hash: blake256(contents),
    };
    let tx = UserTransaction {
        seq: 0,
        network_id: Default::default(),
        action: tx,
    };
    let tx_hash = tx.hash();
    let tx = SignedTransaction {
        signature: ckey::sign(&tx_hash, private),
        signer_public: *public,
        tx,
    };
    Transaction::new("Stamp".to_owned(), serde_cbor::to_vec(&tx).unwrap())
}

pub fn simple1(ctx: &RwLock<Context>) {
    for stateful in ctx.write().statefuls.values_mut() {
        stateful.set_storage(ServiceRef::export(Box::new(MockDb::default()) as Box<dyn SubStorageAccess>))
    }

    let user1: Ed25519KeyPair = Random.generate().unwrap();
    let user2: Ed25519KeyPair = Random.generate().unwrap();

    let mut stampers = HashMap::new();
    stampers.insert(user1.public(), 1usize);
    stampers.insert(user2.public(), 0usize);

    ctx.write().init_genesises.get_mut("stamp").unwrap().init_genesis(&serde_cbor::to_vec(&stampers).unwrap());

    let stamp_by_user1 = tx_stamp(user1.public(), user1.private(), "Hello");
    let stamp_by_user2 = tx_stamp(user2.public(), user2.private(), "Hello");

    ctx.write().tx_owners.get_mut("stamp").unwrap().execute_transaction(&stamp_by_user1).unwrap();
    assert!(ctx.write().tx_owners.get_mut("stamp").unwrap().execute_transaction(&stamp_by_user2).is_err());
}
