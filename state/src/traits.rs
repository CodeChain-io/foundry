// Copyright 2018-2020 Kodebox, Inc.
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

use crate::{Account, ActionData, CacheableItem, Metadata, Module, ModuleDatum, ShardText, StateDB, StateResult};
use ckey::Address;
use ctypes::transaction::ShardTransaction;
use ctypes::{BlockNumber, CommonParams, ShardId, StorageId, Tracker, TxHash};
use merkle_trie::Result as TrieResult;
use primitives::{Bytes, H256};

pub trait TopStateView {
    /// Check caches for required data
    /// First searches for account in the local, then the shared cache.
    /// Populates local cache if nothing found.
    fn account(&self, a: &Address) -> TrieResult<Option<Account>>;

    /// Get the seq of account `a`.
    fn seq(&self, a: &Address) -> TrieResult<u64> {
        Ok(self.account(a)?.map_or(0, |account| account.seq()))
    }

    /// Get the balance of account `a`.
    fn balance(&self, a: &Address) -> TrieResult<u64> {
        Ok(self.account(a)?.map_or(0, |account| account.balance()))
    }

    fn account_exists(&self, a: &Address) -> TrieResult<bool> {
        // Bloom filter does not contain empty accounts, so it is important here to
        // check if account exists in the database directly before EIP-161 is in effect.
        Ok(self.account(a)?.is_some())
    }

    fn account_exists_and_not_null(&self, a: &Address) -> TrieResult<bool> {
        Ok(self.account(a)?.map(|a| !a.is_null()).unwrap_or(false))
    }

    fn account_exists_and_has_seq(&self, a: &Address) -> TrieResult<bool> {
        Ok(self.account(a)?.map(|a| a.seq() != 0).unwrap_or(false))
    }

    fn metadata(&self) -> TrieResult<Option<Metadata>>;

    fn module(&self, storage_id: StorageId) -> TrieResult<Option<Module>>;
    fn module_state<'db>(&'db self, storage_id: StorageId) -> TrieResult<Option<Box<dyn ModuleStateView + 'db>>>;

    fn module_root(&self, storage_id: StorageId) -> TrieResult<Option<H256>> {
        Ok(self.module(storage_id)?.map(|module| *module.root()))
    }

    fn action_data(&self, key: &H256) -> TrieResult<Option<ActionData>>;

    fn module_datum(&self, storage_id: StorageId, key: &dyn AsRef<[u8]>) -> TrieResult<Option<ModuleDatum>> {
        match self.module_state(storage_id)? {
            None => Ok(None),
            Some(state) => state.get_datum(key),
        }
    }
}

pub trait ShardStateView {
    /// Get shard text.
    fn text(&self, tracker: Tracker) -> TrieResult<Option<ShardText>>;
}

pub trait ModuleStateView {
    /// Get module datum from the key
    fn get_datum(&self, key: &dyn AsRef<[u8]>) -> TrieResult<Option<ModuleDatum>>;
    /// Check if the key exists
    fn has_key(&self, key: &dyn AsRef<[u8]>) -> TrieResult<bool>;
}

pub trait ShardState {
    fn apply(
        &mut self,
        transaction: &ShardTransaction,
        sender: &Address,
        shard_owners: &[Address],
        approvers: &[Address],
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
    ) -> StateResult<()>;
}

pub trait TopState {
    /// Remove an existing account.
    fn kill_account(&mut self, account: &Address);

    /// Add `incr` to the balance of account `a`.
    fn add_balance(&mut self, a: &Address, incr: u64) -> TrieResult<()>;
    /// Subtract `decr` from the balance of account `a`.
    fn sub_balance(&mut self, a: &Address, decr: u64) -> StateResult<()>;
    /// Subtracts `by` from the balance of `from` and adds it to that of `to`.
    fn transfer_balance(&mut self, from: &Address, to: &Address, by: u64) -> StateResult<()>;

    /// Increment the seq of account `a` by 1.
    fn inc_seq(&mut self, a: &Address) -> TrieResult<()>;

    fn create_module(&mut self) -> StateResult<()>;
    fn set_module_root(&mut self, storage_id: StorageId, new_root: H256) -> StateResult<()>;

    fn increase_term_id(&mut self, last_term_finished_block_num: u64) -> StateResult<()>;

    fn update_action_data(&mut self, key: &H256, data: Bytes) -> StateResult<()>;
    fn remove_action_data(&mut self, key: &H256);

    fn update_params(&mut self, metadata_seq: u64, params: CommonParams) -> StateResult<()>;
    fn update_term_params(&mut self) -> StateResult<()>;
}

pub trait StateWithCache {
    /// Commits our cached account changes into the trie.
    fn commit(&mut self) -> StateResult<H256>;
    fn commit_and_into_db(self) -> StateResult<(StateDB, H256)>;
}
