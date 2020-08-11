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

use ccrypto::blake256;
use ckey::{verify, Ed25519Public as Public, Signature};
use primitives::H256;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, PartialEq, Eq, Debug, Clone, Copy)]
pub struct NetworkId([u8; 2]);

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = std::str::from_utf8(&self.0).expect("network_id a valid utf8 string");
        write!(f, "{}", s)
    }
}

impl Default for NetworkId {
    fn default() -> Self {
        NetworkId([b't', b'c'])
    }
}

pub trait Action: Serialize + std::fmt::Debug {}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedTransaction<T: Action> {
    pub signature: Signature,
    pub signer_public: Public,
    pub tx: PublicTransaction<T>,
}

impl<T: Action> SignedTransaction<T> {
    pub fn verify(&self) -> Result<(), ()> {
        let message = self.tx.hash();
        if verify(&self.signature, &message, &self.signer_public) {
            Ok(())
        } else {
            Err(())
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct PublicTransaction<T: Action> {
    pub network_id: NetworkId,
    pub action: T,
}

impl<T: Action> PublicTransaction<T> {
    pub fn hash(&self) -> H256 {
        let serialized = serde_cbor::to_vec(&self).unwrap();
        blake256(serialized)
    }
}
