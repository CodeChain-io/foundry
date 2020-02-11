// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// Copyright 2020 Kodebox, Inc.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

extern crate codechain_keystore as ckeystore;

mod util;

use ckey::{verify, Ed25519KeyPair as KeyPair, Ed25519Private as Private};
use ckeystore::accounts_dir::RootDiskDirectory;
use ckeystore::{KeyStore, SimpleSecretStore};
use primitives::H256;
use util::TransientDir;

#[test]
fn secret_store_create() {
    let dir = TransientDir::create().unwrap();
    let _ = KeyStore::open(Box::new(dir)).unwrap();
}

#[test]
#[should_panic]
fn secret_store_open_not_existing() {
    let dir = TransientDir::open();
    let _ = KeyStore::open(Box::new(dir)).unwrap();
}

fn random_secret() -> Private {
    Private::random()
}

#[test]
fn secret_store_create_account() {
    let dir = TransientDir::create().unwrap();
    let store = KeyStore::open(Box::new(dir)).unwrap();
    assert_eq!(store.accounts().unwrap().len(), 0);
    assert!(store.insert_account(random_secret(), &"".into()).is_ok());
    assert_eq!(store.accounts().unwrap().len(), 1);
    assert!(store.insert_account(random_secret(), &"".into()).is_ok());
    assert_eq!(store.accounts().unwrap().len(), 2);
}

#[test]
fn secret_store_sign() {
    let dir = TransientDir::create().unwrap();
    let store = KeyStore::open(Box::new(dir)).unwrap();
    assert!(store.insert_account(random_secret(), &"".into()).is_ok());
    let accounts = store.accounts().unwrap();
    assert_eq!(accounts.len(), 1);
    assert!(store.decrypt_account(&accounts[0], &"".into()).is_ok());
    assert!(store.decrypt_account(&accounts[0], &"1".into()).is_err());
}

#[test]
fn secret_store_change_password() {
    let dir = TransientDir::create().unwrap();
    let store = KeyStore::open(Box::new(dir)).unwrap();
    assert!(store.insert_account(random_secret(), &"".into()).is_ok());
    let accounts = store.accounts().unwrap();
    assert_eq!(accounts.len(), 1);
    assert!(store.decrypt_account(&accounts[0], &"".into()).is_ok());
    assert!(store.change_password(&accounts[0], &"".into(), &"1".into()).is_ok());
    assert!(store.decrypt_account(&accounts[0], &"".into()).is_err());
    assert!(store.decrypt_account(&accounts[0], &"1".into()).is_ok());
}

#[test]
fn secret_store_remove_account() {
    let dir = TransientDir::create().unwrap();
    let store = KeyStore::open(Box::new(dir)).unwrap();
    assert!(store.insert_account(random_secret(), &"".into()).is_ok());
    let accounts = store.accounts().unwrap();
    assert_eq!(accounts.len(), 1);
    assert!(store.remove_account(&accounts[0]).is_ok());
    assert_eq!(store.accounts().unwrap().len(), 0);
    assert!(store.remove_account(&accounts[0]).is_err());
}

fn pat_path() -> &'static str {
    match ::std::fs::metadata("keystore") {
        Ok(_) => "keystore/tests/res/pat",
        Err(_) => "tests/res/pat",
    }
}

fn ciphertext_path() -> &'static str {
    match ::std::fs::metadata("keystore") {
        Ok(_) => "keystore/tests/res/ciphertext",
        Err(_) => "tests/res/ciphertext",
    }
}

#[test]
fn secret_store_load_pat_files() {
    let dir = RootDiskDirectory::at(pat_path());
    let store = KeyStore::open(Box::new(dir)).unwrap();
    assert_eq!(store.accounts().unwrap(), vec![
        "0x3fc74504d2b491d73079975e302279540bf6e44e".into(),
        "0x41178717678e402bdb663d98fe47669d93b29603".into()
    ]);
}

#[test]
fn decrypting_files_with_short_ciphertext() {
    // 0x0e8d3d2a8c5ad882331c94249806bdc2867ca186
    let kp1 =
        KeyPair::from_private("f52e5b0b80c5e7b6fcd64869dd5f4dc84763395b05620209fcc11e7436f0ac05800a29dbeab141ada7923517e945bf4594917473809547bc0bb2e47cd39ac94b".parse().unwrap())
            .unwrap();
    // 0x1c593662f2812124a3ab842faf20acfbf9217eb7
    let kp2 =
        KeyPair::from_private("44795b6f434fde613af66cb01fe14ecf51f0f610b7db38ce3b369e15a49016709f3f180b63b95559a95735385e35cd973c3d4e9f81bbf0faa61cf6159841feb5".parse().unwrap())
            .unwrap();
    let dir = RootDiskDirectory::at(ciphertext_path());
    let store = KeyStore::open(Box::new(dir)).unwrap();
    let accounts = store.accounts().unwrap();
    assert_eq!(accounts, vec![
        "0x0e8d3d2a8c5ad882331c94249806bdc2867ca186".into(),
        "0x1c593662f2812124a3ab842faf20acfbf9217eb7".into()
    ]);

    let message = H256::random();

    let s1 = store.decrypt_account(&accounts[0], &"password".into()).unwrap().sign(&message).unwrap();
    let s2 = store.decrypt_account(&accounts[1], &"password".into()).unwrap().sign(&message).unwrap();
    assert!(verify(&s1, &message, kp1.public()));
    assert!(verify(&s2, &message, kp2.public()));
}
