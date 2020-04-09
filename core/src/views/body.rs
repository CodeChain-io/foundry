// Copyright 2018-2019 Kodebox, Inc.
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

use crate::transaction::LocalizedTransaction;
use crate::Evidence;
use ccrypto::blake256;
use coordinator::validator::Transaction;
use ctypes::{BlockHash, BlockNumber, TxHash};
use rlp::Rlp;

/// View onto block rlp.
pub struct BodyView<'a> {
    rlp: Rlp<'a>,
}

impl<'a> BodyView<'a> {
    /// Creates new view onto block from raw bytes.
    pub fn new(bytes: &'a [u8]) -> BodyView<'a> {
        BodyView {
            rlp: Rlp::new(bytes),
        }
    }

    /// Creates new view onto block from rlp.
    pub fn new_from_rlp(rlp: Rlp<'a>) -> BodyView<'a> {
        BodyView {
            rlp,
        }
    }

    /// Return reference to underlaying rlp.
    pub fn rlp(&self) -> &Rlp<'a> {
        &self.rlp
    }

    /// Return List of evidences in given block.
    pub fn evidences(&self) -> Vec<Evidence> {
        self.rlp.list_at(0).unwrap()
    }

    /// Return number of evidenecs in given block, without deserializing them.
    pub fn evidences_count(&self) -> usize {
        self.rlp.at(0).unwrap().item_count().unwrap()
    }

    /// Return List of transactions in given block.
    pub fn transactions(&self) -> Vec<Transaction> {
        self.rlp.list_at(1).unwrap()
    }

    /// Return List of transactions with additional localization info.
    pub fn localized_transactions(
        &self,
        block_hash: &BlockHash,
        block_number: BlockNumber,
    ) -> Vec<LocalizedTransaction> {
        self.transactions()
            .into_iter()
            .enumerate()
            .map(|(transaction_index, tx)| LocalizedTransaction {
                tx,
                block_hash: *block_hash,
                block_number,
                transaction_index,
            })
            .collect()
    }

    /// Return number of transactions in given block, without deserializing them.
    pub fn transactions_count(&self) -> usize {
        self.rlp.at(1).unwrap().item_count().unwrap()
    }

    /// Return transaction hashes.
    pub fn transaction_hashes(&self) -> Vec<TxHash> {
        self.rlp.at(1).unwrap().iter().map(|rlp| blake256(rlp.as_raw()).into()).collect()
    }

    /// Returns transaction at given index without deserializing unnecessary data.
    pub fn transaction_at(&self, index: usize) -> Option<Transaction> {
        self.rlp.at(1).unwrap().iter().nth(index).map(|rlp| rlp.as_val().unwrap())
    }

    /// Returns localized transaction at given index.
    pub fn localized_transaction_at(
        &self,
        block_hash: &BlockHash,
        block_number: BlockNumber,
        transaction_index: usize,
    ) -> Option<LocalizedTransaction> {
        self.transaction_at(transaction_index).map(|tx| LocalizedTransaction {
            tx,
            block_hash: *block_hash,
            block_number,
            transaction_index,
        })
    }
}
