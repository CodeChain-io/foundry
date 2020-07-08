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

use crate::account;
use crate::common::*;
use crate::token::{self, TokenManager};
use ccrypto::blake256;
use ckey::{Ed25519KeyPair, Generator, KeyPairTrait, Random};
use coordinator::context::SubStorageAccess;
use coordinator::module::*;
use coordinator::types::Transaction;
use parking_lot::RwLock;
use primitives::H256;
use std::collections::HashMap;
use std::sync::Arc;

pub struct MockDb {
    map: HashMap<H256, Vec<u8>>,
}

impl SubStorageAccess for MockDb {
    fn get(&self, key: &dyn AsRef<[u8]>) -> Option<Vec<u8>> {
        self.map.get(&blake256(key)).cloned()
    }

    fn set(&mut self, key: &dyn AsRef<[u8]>, value: Vec<u8>) {
        self.map.insert(blake256(key), value);
    }

    fn remove(&mut self, key: &dyn AsRef<[u8]>) {
        self.map.remove(&blake256(key));
    }

    fn has(&self, key: &dyn AsRef<[u8]>) -> bool {
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

fn setup() -> (Arc<RwLock<account::Context>>, Arc<RwLock<token::Context>>) {
    let db = Arc::new(RwLock::new(MockDb {
        map: Default::default(),
    })) as Arc<RwLock<dyn SubStorageAccess>>;
    let account_module = Arc::new(RwLock::new(account::Context {
        storage: Arc::clone(&db),
    }));
    let account_manager = Arc::clone(&account_module) as Arc<RwLock<dyn account::AccountManager>>;
    let token_module = Arc::new(RwLock::new(token::Context {
        account: Arc::clone(&account_manager),
        storage: Arc::clone(&db),
    }));
    (account_module, token_module)
}

#[test]
fn token_simple1() {
    let (_, token_manager) = setup();

    let issuer1 = blake256("1");

    let user1: Ed25519KeyPair = Random.generate().unwrap();

    token_manager.write().issue_token(&issuer1, user1.public()).unwrap();
    token_manager.write().issue_token(&issuer1, user1.public()).unwrap();
    token_manager.write().issue_token(&issuer1, user1.public()).unwrap();

    assert_eq!(token_manager.read().get_account(user1.public()).unwrap().tokens.len(), 3);
}

#[test]
fn token_simple2() {
    let (_, token_manager) = setup();

    let issuer1 = blake256("1");
    let issuer2 = blake256("2");

    let user1: Ed25519KeyPair = Random.generate().unwrap();
    let user2: Ed25519KeyPair = Random.generate().unwrap();

    token_manager.write().issue_token(&issuer1, user1.public()).unwrap();
    token_manager.write().issue_token(&issuer1, user1.public()).unwrap();
    token_manager.write().issue_token(&issuer2, user1.public()).unwrap();

    let tx = token::Action::TransferToken(token::ActionTransferToken {
        receiver: *user2.public(),
        issuer: issuer1,
    });
    let tx = UserTransaction {
        seq: 0,
        network_id: Default::default(),
        action: tx,
    };
    let tx_hash = tx.hash();
    let tx = SignedTransaction {
        signature: ckey::sign(&tx_hash, user1.private()),
        signer_public: *user1.public(),
        tx,
    };
    let tx = Transaction::new("Token".to_owned(), serde_cbor::to_vec(&tx).unwrap());

    token_manager.write().execute_transaction(&tx).unwrap();

    assert_eq!(token_manager.read().get_account(user1.public()).unwrap().tokens.len(), 2);
    assert_eq!(token_manager.read().get_account(user2.public()).unwrap().tokens.len(), 1);

    let r = token_manager.read().get_owning_accounts_with_issuer(&issuer1).unwrap();
    assert_eq!(r.len(), 2);
    assert!(r.contains(user1.public()));
    assert!(r.contains(user2.public()));

    let r = token_manager.read().get_owning_accounts_with_issuer(&issuer2).unwrap();
    assert_eq!(r.len(), 1);
    assert!(r.contains(user1.public()));
}
