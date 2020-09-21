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

use crate::{ActionData, Metadata, Module, ModuleDatum, StateDB, StateResult};
use ctypes::{CommonParams, ConsensusParams, StorageId};
use merkle_trie::Result as TrieResult;
use primitives::{Bytes, H256};

pub trait TopStateView {
    /// Check caches for required data
    /// First searches for account in the local, then the shared cache.
    /// Populates local cache if nothing found.
    /// Get the seq of account `a`.
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

pub trait ModuleStateView {
    /// Get module datum from the key
    fn get_datum(&self, key: &dyn AsRef<[u8]>) -> TrieResult<Option<ModuleDatum>>;
    /// Check if the key exists
    fn has_key(&self, key: &dyn AsRef<[u8]>) -> TrieResult<bool>;
}

pub trait TopState {
    fn create_module(&mut self) -> StateResult<()>;
    fn set_module_root(&mut self, storage_id: StorageId, new_root: H256) -> StateResult<()>;

    fn increase_term_id(&mut self, last_term_finished_block_num: u64) -> StateResult<()>;

    fn update_action_data(&mut self, key: &H256, data: Bytes) -> StateResult<()>;
    fn remove_action_data(&mut self, key: &H256);

    fn update_params(&mut self, metadata_seq: u64, params: CommonParams) -> StateResult<()>;
    fn update_consensus_params(&mut self, consensus_params: ConsensusParams) -> StateResult<()>;
}

pub trait StateWithCache {
    /// Commits our cached account changes into the trie.
    fn commit(&mut self) -> StateResult<H256>;
    fn commit_and_clone_db(&mut self) -> StateResult<(StateDB, H256)>;
}
