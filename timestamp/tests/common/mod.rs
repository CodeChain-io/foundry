// Copyright 2020 Kodebox, Inc.
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

use ccrypto::blake256;
use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::Coordinator;
use coordinator::Transaction;
use primitives::H256;
use remote_trait_object::ServiceRef;
use std::collections::HashMap;
use timestamp::common::*;

pub fn sign_tx(public: &Public, private: &Private, tx_type: String, action: Vec<u8>) -> Transaction {
    let tx_hash = blake256(&action);
    let tx = SignedTransaction {
        signature: ckey::sign(tx_hash.as_bytes(), private),
        signer_public: *public,
        action,
    };
    Transaction::new(tx_type, serde_cbor::to_vec(&tx).unwrap())
}

pub fn tx_hello(public: &Public, private: &Private, seq: u64) -> Transaction {
    let action = serde_cbor::to_vec(&timestamp::account::TxHello {
        seq,
    })
    .unwrap();
    sign_tx(public, private, "hello".to_owned(), action)
}

pub fn tx_stamp(public: &Public, private: &Private, seq: u64, contents: &str) -> Transaction {
    let action = serde_cbor::to_vec(&timestamp::stamp::TxStamp {
        seq,
        hash: blake256(contents),
    })
    .unwrap();
    sign_tx(public, private, "stamp".to_owned(), action)
}

pub fn tx_token_transfer(public: &Public, private: &Private, seq: u64, receiver: Public, issuer: H256) -> Transaction {
    let action = serde_cbor::to_vec(&timestamp::token::TxTransferToken {
        seq,
        issuer,
        receiver,
    })
    .unwrap();
    sign_tx(public, private, "token-transfer".to_owned(), action)
}

#[derive(Default)]
pub struct TestStorage {
    map: HashMap<Vec<u8>, Vec<u8>>,
}

impl remote_trait_object::Service for TestStorage {}

impl SubStorageAccess for TestStorage {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.map.get(key).map(|x| x.to_owned())
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        self.map.insert(key.to_vec(), value);
    }

    fn has(&self, key: &[u8]) -> bool {
        self.map.get(key).is_some()
    }

    fn remove(&mut self, key: &[u8]) {
        self.map.remove(key);
    }
}

pub fn set_empty_session(id: SessionId, c: &Coordinator) {
    for (_, s) in c.services().stateful.lock().iter_mut() {
        s.new_session(id, ServiceRef::create_export(Box::new(TestStorage::default()) as Box<dyn SubStorageAccess>))
    }
}

pub struct Services<'a> {
    // no stateful here
    pub init_genesis: HashMap<&'a str, &'a dyn InitGenesis>,
    pub genesis_config: HashMap<&'a str, &'a [u8]>,
    pub tx_owner: HashMap<&'a str, &'a dyn TxOwner>,
    pub handle_crimes: &'a dyn HandleCrimes,
    pub init_consensus: &'a dyn InitConsensus,
    pub update_consensus: &'a dyn UpdateConsensus,
    pub tx_sorter: &'a dyn TxSorter,
    pub handle_graphqls: HashMap<&'a str, &'a dyn HandleGraphQlRequest>,
}

impl<'a> Services<'a> {
    pub fn new(c: &'a Coordinator) -> Self {
        let s = c.services();
        Self {
            init_genesis: s.init_genesis.iter().map(|(s, x)| (s.as_str(), x.as_ref())).collect(),
            genesis_config: s.genesis_config.iter().map(|(s, x)| (s.as_str(), x.as_ref())).collect(),
            tx_owner: s.tx_owner.iter().map(|(s, x)| (s.as_str(), x.as_ref())).collect(),
            handle_crimes: s.handle_crimes.as_ref(),
            init_consensus: s.init_consensus.as_ref(),
            update_consensus: s.update_consensus.as_ref(),
            tx_sorter: s.tx_sorter.as_ref(),
            handle_graphqls: s.handle_graphqls.iter().map(|(s, x)| (s.as_str(), x.as_ref())).collect(),
        }
    }
}
