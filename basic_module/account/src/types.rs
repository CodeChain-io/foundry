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
use ckey::{Ed25519Public as Public, NetworkId, Signature};
use primitives::H256;

#[allow(dead_code)]
pub type ErroneousTransactions = Vec<SignedTransaction>;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Account {
    pub balance: u64,
    pub sequence: u64,
}

impl From<Vec<u8>> for Account {
    fn from(vec: Vec<u8>) -> Account {
        serde_cbor::from_slice(&vec).unwrap()
    }
}

impl Default for Account {
    fn default() -> Self {
        Account {
            balance: 0,
            sequence: 0,
        }
    }
}

#[allow(dead_code)]
impl Account {
    pub fn new(balance: u64, sequence: u64) -> Account {
        Account {
            balance,
            sequence,
        }
    }

    pub fn to_vec(&self) -> Vec<u8> {
        serde_cbor::to_vec(&self).unwrap()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Transaction {
    pub seq: u64,
    pub fee: u64,
    pub network_id: NetworkId,
    pub action: Action,
}

impl Transaction {
    pub fn hash(&self) -> H256 {
        let serialized = serde_cbor::to_vec(&self).unwrap();
        blake256(serialized)
    }
}

#[derive(Clone)]
pub struct SignedTransaction {
    pub signature: Signature,
    pub signer_public: Public,
    pub tx: Transaction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
    Pay {
        sender: Public,
        receiver: Public,
        quantity: u64,
    },
}

impl Action {
    pub fn min_fee(&self) -> u64 {
        // Where can we initialize the min fee
        // We need both consensus-defined minimum fee and machine-defined minimum fee
        unimplemented!()
    }
}
