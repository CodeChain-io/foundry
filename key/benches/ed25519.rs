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

extern crate codechain_key as ckey;
extern crate test;

use ckey::{sign, verify, Ed25519KeyPair, Generator, KeyPairTrait, Message, Random};
use test::Bencher;

#[bench]
fn ec25519_sign(b: &mut Bencher) {
    b.iter(|| {
        let key_pair: Ed25519KeyPair = Random.generate().unwrap();
        let message = Message::random();
        let _signature = sign(message.as_ref(), key_pair.private());
    })
}

#[bench]
fn ed25519_sign_and_verify(b: &mut Bencher) {
    b.iter(|| {
        let key_pair: Ed25519KeyPair = Random.generate().unwrap();
        let message = Message::random();
        let signature = sign(message.as_ref(), key_pair.private());
        assert!(verify(&signature, message.as_ref(), key_pair.public()));
    })
}
