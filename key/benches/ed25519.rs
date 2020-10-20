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

extern crate codechain_key as ckey;

use ckey::{sign, verify, Ed25519KeyPair, Generator, KeyPairTrait, Message, Random};
use criterion::{criterion_group, criterion_main, Criterion};

fn ec25519_sign(c: &mut Criterion) {
    c.bench_function("ec25519_sign", |b| {
        b.iter(|| {
            let key_pair: Ed25519KeyPair = Random.generate().unwrap();
            let message = Message::random();
            let _signature = sign(message.as_ref(), key_pair.private());
        })
    });
}

fn ed25519_sign_and_verify(c: &mut Criterion) {
    c.bench_function("ed25519_sign_and_verify", |b| {
        b.iter(|| {
            let key_pair: Ed25519KeyPair = Random.generate().unwrap();
            let message = Message::random();
            let signature = sign(message.as_ref(), key_pair.private());
            assert!(verify(&signature, message.as_ref(), key_pair.public()));
        })
    });
}

criterion_group!(benches, ec25519_sign, ed25519_sign_and_verify);
criterion_main!(benches);
