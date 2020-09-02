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
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::Transaction;
use primitives::H256;
use remote_trait_object::Service;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct TxStamp {
    pub hash: H256,
}

impl Action for TxStamp {}

pub type OwnTransaction = crate::common::SignedTransaction<TxStamp>;

pub struct GetAccountAndSeq;

impl Service for GetAccountAndSeq {}

impl crate::sorting::GetAccountAndSeq for GetAccountAndSeq {
    fn get_account_and_seq(&self, tx: &Transaction) -> Result<(Public, TxSeq), ()> {
        assert_eq!(tx.tx_type(), "stamp");
        let tx: OwnTransaction = serde_cbor::from_slice(&tx.body()).map_err(|_| ())?;
        Ok((tx.signer_public, tx.tx.seq))
    }
}
