// Copyright 2018-2019 Kodebox, Inc.
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

use super::TransactionView;
use crate::transaction::{LocalizedTransaction, UnverifiedTransaction};
use ccrypto::blake256;
use ctypes::{BlockHash, BlockNumber, TransactionIndex, TxHash};
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

    /// Return List of transactions in given block.
    pub fn transactions(&self) -> Vec<UnverifiedTransaction> {
        self.rlp.list_at(0).unwrap()
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
            .map(|(transaction_index, signed)| LocalizedTransaction {
                signed,
                block_hash: *block_hash,
                block_number,
                transaction_index: transaction_index as TransactionIndex,
                cached_signer_public: None,
            })
            .collect()
    }

    /// Return number of transactions in given block, without deserializing them.
    pub fn transactions_count(&self) -> usize {
        self.rlp.at(0).unwrap().item_count().unwrap()
    }

    /// Return List of transactions in given block.
    pub fn transaction_views(&self) -> Vec<TransactionView<'a>> {
        self.rlp.at(0).unwrap().iter().map(TransactionView::new_from_rlp).collect()
    }

    /// Return transaction hashes.
    pub fn transaction_hashes(&self) -> Vec<TxHash> {
        self.rlp.at(0).unwrap().iter().map(|rlp| blake256(rlp.as_raw()).into()).collect()
    }

    /// Returns transaction at given index without deserializing unnecessary data.
    pub fn transaction_at(&self, index: TransactionIndex) -> Option<UnverifiedTransaction> {
        self.rlp.at(0).unwrap().iter().nth(index as usize).map(|rlp| rlp.as_val().unwrap())
    }

    /// Returns localized transaction at given index.
    pub fn localized_transaction_at(
        &self,
        block_hash: &BlockHash,
        block_number: BlockNumber,
        transaction_index: TransactionIndex,
    ) -> Option<LocalizedTransaction> {
        self.transaction_at(transaction_index).map(|signed| LocalizedTransaction {
            signed,
            block_hash: *block_hash,
            block_number,
            transaction_index,
            cached_signer_public: None,
        })
    }
}
