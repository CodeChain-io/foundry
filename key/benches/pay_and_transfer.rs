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

#![feature(test)]

extern crate codechain_crypto as ccrypto;
extern crate codechain_key as ckey;
extern crate test;

use ccrypto::Blake;
use ckey::{sign, verify, Ed25519KeyPair, Generator, KeyPairTrait, Message, Random};
use primitives::{H160, H256};
use test::Bencher;

#[bench]
fn pay_with_ed25519(b: &mut Bencher) {
    // A transaction only has a signature.
    let key_pair: Ed25519KeyPair = Random.generate().unwrap();
    let transaction = Message::random();
    let transaction_hash: H256 = Blake::blake(transaction);
    let signature = sign(transaction_hash.as_ref(), key_pair.private());
    b.iter(|| {
        let transaction_hash: H256 = Blake::blake(transaction);
        assert!(verify(&signature, transaction_hash.as_ref(), key_pair.public()));
    });
}

#[bench]
fn transfer_with_ed25519(b: &mut Bencher) {
    // Assuming 2-input transfer transaction.
    let key_pair_0: Ed25519KeyPair = Random.generate().unwrap();
    let key_pair_1: Ed25519KeyPair = Random.generate().unwrap();
    let key_pair_2: Ed25519KeyPair = Random.generate().unwrap();

    let transaction = Message::random();
    let transaction_hash: H256 = Blake::blake(transaction);
    let signature_tx = sign(transaction_hash.as_ref(), key_pair_0.private());
    let signature_1 = sign(transaction_hash.as_ref(), key_pair_1.private());
    let signature_2 = sign(transaction_hash.as_ref(), key_pair_2.private());

    let lock_script_1 = Message::random();
    let lock_script_hash_1: H160 = Blake::blake(lock_script_1);
    let lock_script_2 = Message::random();
    let lock_script_hash_2: H160 = Blake::blake(lock_script_2);

    b.iter(|| {
        // Transaction verification
        let transaction_hash: H256 = Blake::blake(transaction);
        assert!(verify(&signature_tx, transaction_hash.as_ref(), key_pair_0.public()));

        // Input 1 verification
        // Lock script hash check
        assert_eq!(lock_script_hash_1, Blake::blake(lock_script_1));
        // Unfortunately, hash again because of partial hashing
        let transaction_hash_1: H256 = Blake::blake(transaction);
        assert!(verify(&signature_1, transaction_hash_1.as_ref(), key_pair_1.public()));

        // Input 2 verification
        assert_eq!(lock_script_hash_2, Blake::blake(lock_script_2));
        let transaction_hash_2: H256 = Blake::blake(transaction);
        assert!(verify(&signature_2, transaction_hash_2.as_ref(), key_pair_2.public()));
    });
}
