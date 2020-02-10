// Copyright 2018 Kodebox, Inc.
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

#![feature(test)]

extern crate codechain_key as ckey;
extern crate test;

use ckey::{
    aggregate_signatures_bls, sign_bls, verify_aggregated_bls, verify_bls, BlsPublic, BlsSignature, Generator, Message,
    Random,
};
use test::Bencher;

#[bench]
fn bls_sign(b: &mut Bencher) {
    b.iter(|| {
        let key_pair = Random.generate_bls().unwrap();
        let message = Message::random();
        let _signature = sign_bls(key_pair.private(), &message);
    });
}

#[bench]
fn bls_sign_and_verify(b: &mut Bencher) {
    b.iter(|| {
        let key_pair = Random.generate_bls().unwrap();
        let message = Message::random();
        let signature = sign_bls(key_pair.private(), &message);
        assert_eq!(Ok(true), verify_bls(key_pair.public(), &signature, &message));
    });
}

#[bench]
fn bls_aggregate_signatures(b: &mut Bencher) {
    let num_validators = 30;
    b.iter(|| {
        let key_pairs: Vec<_> = (0..num_validators).map(|_| Random.generate_bls().unwrap()).collect();
        let message = Message::random();
        let signatures: Vec<BlsSignature> =
            key_pairs.iter().map(|key_pair| sign_bls(key_pair.private(), &message)).collect();
        let _aggregated_signature = aggregate_signatures_bls(&signatures);
    })
}

#[bench]
fn bls_aggregate_and_verify(b: &mut Bencher) {
    let num_validators = 30;
    b.iter(|| {
        let key_pairs: Vec<_> = (0..num_validators).map(|_| Random.generate_bls().unwrap()).collect();
        let message = Message::random();
        let signatures: Vec<BlsSignature> =
            key_pairs.iter().map(|key_pair| sign_bls(key_pair.private(), &message)).collect();
        let aggregated_signature = aggregate_signatures_bls(&signatures);
        let publics: Vec<BlsPublic> = key_pairs.iter().map(|key_pair| *key_pair.public()).collect();
        assert_eq!(Ok(true), verify_aggregated_bls(&publics, &aggregated_signature, &message))
    })
}
