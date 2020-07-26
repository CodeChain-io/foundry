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

use super::Action;
use ccore::{LocalizedTransaction, PendingVerifiedTransactions, VerifiedTransaction};
use cjson::uint::Uint;
use ckey::{NetworkId, Signature};
use ctypes::{BlockHash, TransactionIndex, TxHash};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {
    pub block_number: Option<u64>,
    pub block_hash: Option<BlockHash>,
    pub transaction_index: Option<TransactionIndex>,
    pub result: Option<bool>,
    pub network_id: NetworkId,
    pub action: Action,
    pub hash: TxHash,
    pub sig: Signature,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingTransactions {
    transactions: Vec<Transaction>,
    last_timestamp: Option<u64>,
}

impl From<PendingVerifiedTransactions> for PendingTransactions {
    fn from(p: PendingVerifiedTransactions) -> Self {
        let transactions = p.transactions.into_iter().map(From::from).collect();
        Self {
            transactions,
            last_timestamp: p.last_timestamp,
        }
    }
}

impl From<LocalizedTransaction> for Transaction {
    fn from(p: LocalizedTransaction) -> Self {
        let sig = p.unverified_tx().signature();
        Self {
            block_number: Some(p.block_number),
            block_hash: Some(p.block_hash),
            transaction_index: Some(p.transaction_index),
            result: Some(true),
            network_id: p.unverified_tx().transaction().network_id,
            action: Action::from_core(
                p.unverified_tx().transaction().action.clone(),
                p.unverified_tx().transaction().network_id,
            ),
            hash: p.unverified_tx().hash(),
            sig,
        }
    }
}

impl From<VerifiedTransaction> for Transaction {
    fn from(p: VerifiedTransaction) -> Self {
        let sig = p.signature();
        Self {
            block_number: None,
            block_hash: None,
            transaction_index: None,
            result: None,
            network_id: p.transaction().network_id,
            action: Action::from_core(p.transaction().action.clone(), p.transaction().network_id),
            hash: p.hash(),
            sig,
        }
    }
}
