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

use coordinator::Transaction;
use ctypes::{BlockHash, BlockNumber, TransactionIndex};

pub struct PendingTransactions {
    pub transactions: Vec<Transaction>,
    pub last_timestamp: Option<u64>,
}

/// Signed Transaction that is a part of canon blockchain.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalizedTransaction {
    /// transaction
    pub tx: Transaction,
    /// Block number.
    pub block_number: BlockNumber,
    /// Block hash.
    pub block_hash: BlockHash,
    /// Transaction index within block.
    pub transaction_index: TransactionIndex,
}

impl LocalizedTransaction {
    pub fn tx(&self) -> &Transaction {
        &self.tx
    }
}

impl From<LocalizedTransaction> for Transaction {
    fn from(localized_tx: LocalizedTransaction) -> Self {
        localized_tx.tx
    }
}
