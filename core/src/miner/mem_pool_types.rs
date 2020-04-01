// Copyright 2019-2020 Kodebox, Inc.
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

use coordinator::{Transaction, TxOrigin};
use ctypes::{BlockNumber, TxHash};
use std::collections::HashMap;

/// Point in time when transaction was inserted
pub type PoolingInstant = BlockNumber;

/// Transaction item in the mem pool.
#[derive(Clone, Eq, PartialEq, Debug, RlpEncodable)]
pub struct MemPoolItem {
    /// Transaction.
    pub tx: Transaction,
    /// Transaction origin.
    pub origin: TxOrigin,
    /// Insertion time
    pub inserted_block_number: PoolingInstant,
    /// Insertion timstamp
    pub inserted_timestamp: u64,
    /// ID assigned upon insertion, should be unique.
    pub insertion_id: u64,
    /// Memory usage of this transaction.
    /// Currently using the RLP byte length of the transaction as the mem usage.
    pub mem_usage: usize,
}

impl MemPoolItem {
    pub fn new(
        tx: Transaction,
        origin: TxOrigin,
        inserted_block_number: PoolingInstant,
        inserted_timestamp: u64,
        insertion_id: u64,
    ) -> Self {
        let mem_usage = rlp::encode(&tx).len();
        ctrace!(MEM_POOL, "New tx with {} bytes", mem_usage);
        MemPoolItem {
            tx,
            origin,
            inserted_block_number,
            inserted_timestamp,
            insertion_id,
            mem_usage,
        }
    }

    pub fn hash(&self) -> TxHash {
        self.tx.hash()
    }
}

#[derive(Debug, PartialEq)]
pub struct TransactionPool {
    pub pool: HashMap<TxHash, MemPoolItem>,
    /// Memory usage of the transactions in the queue
    pub mem_usage: usize,
    /// Count of the external transactions in the queue
    pub count: usize,
}

impl TransactionPool {
    pub fn new() -> Self {
        Self {
            pool: Default::default(),
            mem_usage: 0,
            count: 0,
        }
    }

    pub fn clear(&mut self) {
        self.pool.clear();
        self.mem_usage = 0;
        self.count = 0;
    }

    pub fn len(&self) -> usize {
        self.pool.len()
    }

    pub fn insert(&mut self, item: MemPoolItem) {
        if !item.origin.is_local() {
            self.mem_usage += item.mem_usage;
            self.count += 1;
        }
        self.pool.insert(item.hash(), item);
    }

    pub fn contains(&self, hash: &TxHash) -> bool {
        self.pool.contains_key(hash)
    }

    pub fn remove(&mut self, hash: &TxHash) -> bool {
        if let Some(item) = self.pool.remove(hash) {
            if !item.origin.is_local() {
                self.mem_usage -= item.mem_usage;
                self.count -= 1;
            }
            true
        } else {
            false
        }
    }
}
