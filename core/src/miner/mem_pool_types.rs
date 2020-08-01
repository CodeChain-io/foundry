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

use coordinator::TransactionWithMetadata;
use ctypes::TxHash;
use std::collections::HashMap;

#[derive(Debug, PartialEq)]
pub struct TransactionPool {
    pub pool: HashMap<TxHash, TransactionWithMetadata>,
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

    pub fn insert(&mut self, item: TransactionWithMetadata) {
        if !item.origin.is_local() {
            self.mem_usage += item.size();
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
                self.mem_usage -= item.size();
                self.count -= 1;
            }
            true
        } else {
            false
        }
    }
}
