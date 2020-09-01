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

use crate::common::*;
use ccrypto::blake256;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::Transaction;
use primitives::H256;
use remote_trait_object::Service;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Token {
    /// The issuer is recorded in the Token.
    /// Since Token module is general, it can be used from various other modules.
    /// `issuer` is for preventing different tokens to get mixed in such case.
    ///
    /// Even in a same module, you could consider advanced scheme where you
    /// distribute tokens with various issuer for special purpose (e.g invalidatablity)
    pub issuer: H256,
}

#[derive(Serialize, Deserialize, Default, Debug)]
pub struct Account {
    pub tokens: Vec<Token>,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum Error {
    NoSuchAccount,
    InvalidKey,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ActionTransferToken {
    pub receiver: Public,

    /// There is no difference for tokens as far as the issuer is same;
    /// Thus it is enough to speicfy which token to transfer only by the issuer.
    pub issuer: H256,
}
impl Action for ActionTransferToken {}
pub type OwnTransaction = SignedTransaction<ActionTransferToken>;

pub struct GetAccountAndSeq;
impl Service for GetAccountAndSeq {}
impl crate::sorting::GetAccountAndSeq for GetAccountAndSeq {
    fn get_account_and_seq(&self, tx: &Transaction) -> Result<(Public, TxSeq), ()> {
        assert_eq!(tx.tx_type(), "token");
        let tx: OwnTransaction = serde_cbor::from_slice(&tx.body()).map_err(|_| ())?;
        Ok((tx.signer_public, tx.tx.seq))
    }
}

pub fn get_state_key(public: &Public) -> H256 {
    blake256(&{
        let mut v = serde_cbor::to_vec(&public).unwrap();
        v.extend_from_slice(b"Token-Module-Account");
        v
    } as &[u8])
}

pub fn get_state_key_account_set(issuer: &H256) -> H256 {
    blake256(&{
        let mut v = serde_cbor::to_vec(&issuer).unwrap();
        v.extend_from_slice(b"Token-Module-Account-Set");
        v
    } as &[u8])
}
