// Copyright 2019-2020 Kodebox, Inc.
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

use coordinator::validator::TransactionWithMetadata;
use ctypes::transaction::Action;
use ctypes::{BlockNumber, TxHash};
use std::collections::HashMap;

pub type PoolingInstant = BlockNumber;

#[derive(Debug, PartialEq)]
pub struct TransactionPool {
    /// Priority queue for transactions
    pub pool: HashMap<TxHash, TransactionWithMetadata>,
    /// Memory usage of the transactions in the queue
    pub mem_usage: usize,
    /// Count of the external transactions in the queue
    pub count: usize,
}

impl<'a> TransactionPool {
    pub fn new() -> Self {
        Self {
            pool: HashMap::new(),
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

    pub fn contains(&self, hash: &TxHash) -> bool {
        self.pool.contains_key(hash)
    }

    pub fn insert(&mut self, tx: TransactionWithMetadata) {
        self.pool.insert(tx.hash(), tx);
        self.mem_usage += tx.size();
        self.count += 1;
    }

    pub fn remove(&mut self, hash: &TxHash) {
        assert!(self.contains(hash));
        self.pool.remove(hash);
    }
}

/// Current status of the pool
#[derive(Debug)]
pub struct MemPoolStatus {
    /// Number of pending transactions (ready to go to block)
    pub pending: usize,
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
/// Minimum fee thresholds defined not by network but by Mempool
pub struct MemPoolMinFees {
    pub min_pay_transaction_cost: u64,
    pub min_create_shard_transaction_cost: u64,
    pub min_set_shard_owners_transaction_cost: u64,
    pub min_set_shard_users_transaction_cost: u64,
    pub min_wrap_ccc_transaction_cost: u64,
    pub min_custom_transaction_cost: u64,
    pub min_asset_mint_cost: u64,
    pub min_asset_transfer_cost: u64,
    pub min_asset_scheme_change_cost: u64,
    pub min_asset_supply_increase_cost: u64,
    pub min_asset_unwrap_ccc_cost: u64,
}

impl MemPoolMinFees {
    #[allow(clippy::too_many_arguments)]
    pub fn create_from_options(
        min_pay_cost_option: Option<u64>,
        min_create_shard_cost_option: Option<u64>,
        min_set_shard_owners_cost_option: Option<u64>,
        min_set_shard_users_cost_option: Option<u64>,
        min_wrap_ccc_cost_option: Option<u64>,
        min_custom_cost_option: Option<u64>,
        min_asset_mint_cost_option: Option<u64>,
        min_asset_transfer_cost_option: Option<u64>,
        min_asset_scheme_change_cost_option: Option<u64>,
        min_asset_supply_increase_cost_option: Option<u64>,
        min_asset_unwrap_ccc_cost_option: Option<u64>,
    ) -> Self {
        MemPoolMinFees {
            min_pay_transaction_cost: min_pay_cost_option.unwrap_or_default(),
            min_create_shard_transaction_cost: min_create_shard_cost_option.unwrap_or_default(),
            min_set_shard_owners_transaction_cost: min_set_shard_owners_cost_option.unwrap_or_default(),
            min_set_shard_users_transaction_cost: min_set_shard_users_cost_option.unwrap_or_default(),
            min_wrap_ccc_transaction_cost: min_wrap_ccc_cost_option.unwrap_or_default(),
            min_custom_transaction_cost: min_custom_cost_option.unwrap_or_default(),
            min_asset_mint_cost: min_asset_mint_cost_option.unwrap_or_default(),
            min_asset_transfer_cost: min_asset_transfer_cost_option.unwrap_or_default(),
            min_asset_scheme_change_cost: min_asset_scheme_change_cost_option.unwrap_or_default(),
            min_asset_supply_increase_cost: min_asset_supply_increase_cost_option.unwrap_or_default(),
            min_asset_unwrap_ccc_cost: min_asset_unwrap_ccc_cost_option.unwrap_or_default(),
        }
    }
    pub fn min_cost(&self, action: &Action) -> u64 {
        match action {
            Action::Pay {
                ..
            } => self.min_pay_transaction_cost,
            Action::CreateShard {
                ..
            } => self.min_create_shard_transaction_cost,
            Action::SetShardOwners {
                ..
            } => self.min_set_shard_owners_transaction_cost,
            Action::SetShardUsers {
                ..
            } => self.min_set_shard_users_transaction_cost,
            Action::Custom {
                ..
            } => self.min_custom_transaction_cost,
            Action::ShardStore {
                ..
            } => {
                // FIXME
                0
            }
        }
    }
}
