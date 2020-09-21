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

// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.
//
// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! A mutable state representation suitable to execute transactions.
//! Generic over a `Backend`. Deals with `Account`s.
//! Unconfirmed sub-states are managed with `checkpoint`s which may be canonicalized
//! or rolled back.

use crate::cache::{ModuleCache, TopCache};
use crate::checkpoint::{CheckpointId, StateWithCheckpoint};
use crate::traits::{ModuleStateView, StateWithCache, TopState, TopStateView};
use crate::{
    ActionData, Metadata, MetadataAddress, Module, ModuleAddress, ModuleDatum, ModuleDatumAddress, ModuleLevelState,
    StateDB, StateResult,
};
use cdb::{AsHashDB, DatabaseError};
use coordinator::context::{StorageAccess, SubStorageAccess};
use ctypes::errors::RuntimeError;
use ctypes::util::unexpected::Mismatch;
use ctypes::{CommonParams, ConsensusParams, StorageId};
use kvdb::DBTransaction;
use merkle_trie::{Result as TrieResult, TrieError, TrieFactory};
use parking_lot::{Mutex, RwLock};
use primitives::{Bytes, H256};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;
use std::sync::Arc;

/// Representation of the entire state of all accounts in the system.
///
/// Local cache contains changes made locally and changes accumulated
/// locally from previous commits.
///
/// **** IMPORTANT *************************************************************
/// All the modifications to the account data must set the `Dirty` state in the
/// `Entry<Item>`. This is done in `require` and `require_or_from`. So just
/// use that.
/// ****************************************************************************
///
/// State checkpointing.
///
/// A new checkpoint can be created with `checkpoint()`. checkpoints can be
/// created in a hierarchy.
/// When a checkpoint is active all changes are applied directly into
/// `cache` and the original value is copied into an active checkpoint.
/// Reverting a checkpoint with `revert_to_checkpoint` involves copying
/// original values from the latest checkpoint back into `cache`. The code
/// takes care not to overwrite cached storage while doing that.
/// checkpoint can be discarded with `discard_checkpoint`. All of the orignal
/// backed-up values are moved into a parent checkpoint (if any).
pub struct TopLevelState {
    db: Arc<RwLock<StateDB>>,
    root: H256,

    top_cache: TopCache,
    module_caches: RefCell<HashMap<StorageId, Arc<Mutex<ModuleCache>>>>,
    id_of_checkpoints: Vec<CheckpointId>,
}

impl TopStateView for TopLevelState {
    /// Check caches for required data
    /// First searches for account in the local, then the shared cache.
    /// Populates local cache if nothing found.
    fn metadata(&self) -> TrieResult<Option<Metadata>> {
        let db = self.db.read();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let address = MetadataAddress::new();
        self.top_cache.metadata(&address, &trie)
    }

    fn module(&self, storage_id: StorageId) -> TrieResult<Option<Module>> {
        let db = self.db.read();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let module_address = ModuleAddress::new(storage_id);
        self.top_cache.module(&module_address, &trie)
    }

    fn module_state(&self, storage_id: StorageId) -> TrieResult<Option<Box<dyn ModuleStateView>>> {
        match self.module_root(storage_id)? {
            Some(module_root) => {
                let mut module_caches = self.module_caches.borrow_mut();
                let module_cache = module_caches.entry(storage_id).or_default();
                Ok(Some(Box::new(ModuleLevelState::from_existing(
                    storage_id,
                    Arc::clone(&self.db),
                    module_root,
                    Arc::clone(module_cache),
                )?)))
            }
            None => Ok(None),
        }
    }

    fn action_data(&self, key: &H256) -> TrieResult<Option<ActionData>> {
        let db = self.db.read();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        Ok(self.top_cache.action_data(key, &trie)?.map(Into::into))
    }
}

impl StorageAccess for TopLevelState {
    fn sub_storage(&mut self, storage_id: u16) -> Box<dyn SubStorageAccess> {
        match self.module_state_mut(storage_id) {
            Ok(sub_storage) => Box::new(sub_storage),
            Err(e) => panic!("unknown sub-storage: {}", e),
        }
    }

    fn create_checkpoint(&mut self) {
        StateWithCheckpoint::create_checkpoint(self, TOP_CHECKPOINT)
    }

    fn revert_to_the_checkpoint(&mut self) {
        StateWithCheckpoint::revert_to_checkpoint(self, TOP_CHECKPOINT)
    }

    fn discard_checkpoint(&mut self) {
        StateWithCheckpoint::discard_checkpoint(self, TOP_CHECKPOINT)
    }
}

impl StateWithCache for TopLevelState {
    fn commit(&mut self) -> StateResult<H256> {
        let module_changes = self
            .module_caches
            .borrow()
            .keys()
            .map(|storage_id| {
                if let Some(module_root) = self.module_root(*storage_id)? {
                    Ok(Some((*storage_id, module_root)))
                } else {
                    Ok(None)
                }
            })
            .collect::<StateResult<Vec<_>>>()?;
        for (storage_id, mut module_root) in module_changes.into_iter().filter_map(|result| result) {
            {
                let mut db = self.db.write();
                let mut trie = TrieFactory::from_existing(db.as_hashdb_mut(), &mut module_root)?;

                let module_caches = self.module_caches.borrow();
                let module_cache = module_caches.get(&storage_id).expect("Module must exist");

                module_cache.lock().commit(&mut trie)?;
            }
            self.set_module_root(storage_id, module_root)?;
        }
        {
            let mut db = self.db.write();
            let mut trie = TrieFactory::from_existing(db.as_hashdb_mut(), &mut self.root)?;
            self.top_cache.commit(&mut trie)?;
        }
        Ok(self.root)
    }

    fn commit_and_clone_db(&mut self) -> StateResult<(StateDB, H256)> {
        let root = self.commit()?;
        Ok((self.db.read().clone(&root), root))
    }
}

const TOP_CHECKPOINT: CheckpointId = 777;

impl StateWithCheckpoint for TopLevelState {
    fn create_checkpoint(&mut self, id: CheckpointId) {
        ctrace!(STATE, "Checkpoint({}) for top level is created", id);
        self.id_of_checkpoints.push(id);
        self.top_cache.checkpoint();

        self.module_caches.borrow_mut().values_mut().for_each(|cache| cache.lock().checkpoint())
    }

    fn discard_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is discarded", id);
        self.top_cache.discard_checkpoint();

        self.module_caches.borrow_mut().values_mut().for_each(|cache| cache.lock().discard_checkpoint())
    }

    fn revert_to_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is reverted", id);
        self.top_cache.revert_to_checkpoint();

        self.module_caches.borrow_mut().values_mut().for_each(|cache| cache.lock().revert_to_checkpoint())
    }
}

impl TopLevelState {
    /// Creates new state with existing state root
    pub fn from_existing(db: StateDB, root: H256) -> Result<Self, TrieError> {
        if !db.as_hashdb().contains(&root) {
            return Err(TrieError::InvalidStateRoot(root))
        }

        let top_cache = db.top_cache();
        let module_caches =
            db.module_caches().into_iter().map(|(id, cache)| (id, Arc::new(Mutex::new(cache)))).collect();

        let state = TopLevelState {
            db: Arc::new(RwLock::new(db)),
            root,
            top_cache,
            module_caches: RefCell::new(module_caches),
            id_of_checkpoints: Default::default(),
        };

        Ok(state)
    }

    fn create_module_level_state(&mut self, storage_id: StorageId) -> StateResult<()> {
        const DEFAULT_MODULE_ROOT: H256 = ccrypto::BLAKE_NULL_RLP;
        {
            let mut module_caches = self.module_caches.borrow_mut();
            let module_cache = module_caches.entry(storage_id).or_default();
            ModuleLevelState::from_existing(
                storage_id,
                Arc::clone(&self.db),
                DEFAULT_MODULE_ROOT,
                Arc::clone(&module_cache),
            )?;
        }
        ctrace!(STATE, "module storage({}) created", storage_id);
        self.set_module_root(storage_id, DEFAULT_MODULE_ROOT)?;
        Ok(())
    }

    fn module_state_mut(&self, storage_id: StorageId) -> StateResult<ModuleLevelState> {
        let module_root = self.module_root(storage_id)?.ok_or_else(|| RuntimeError::InvalidStorageId(storage_id))?;
        let mut module_caches = self.module_caches.borrow_mut();
        let module_cache = module_caches.entry(storage_id).or_default();
        Ok(ModuleLevelState::from_existing(storage_id, Arc::clone(&self.db), module_root, Arc::clone(&module_cache))?)
    }

    pub fn get_metadata_mut(&self) -> TrieResult<RefMut<'_, Metadata>> {
        let db = self.db.read();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let address = MetadataAddress::new();
        self.top_cache.metadata_mut(&address, &trie)
    }

    fn get_module_mut(&self, storage_id: StorageId) -> TrieResult<RefMut<'_, Module>> {
        let db = self.db.read();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let address = ModuleAddress::new(storage_id);
        self.top_cache.module_mut(&address, &trie)
    }

    fn get_action_data_mut(&self, key: &H256) -> TrieResult<RefMut<'_, ActionData>> {
        let db = self.db.read();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.action_data_mut(key, &trie)
    }

    pub fn journal_under(&self, batch: &mut DBTransaction, now: u64) -> Result<u32, DatabaseError> {
        self.db.write().journal_under(batch, now, self.root)
    }

    pub fn top_cache(&self) -> &TopCache {
        &self.top_cache
    }

    pub fn cached_module_data(&self) -> Vec<(ModuleDatumAddress, Option<ModuleDatum>)> {
        let mut flatten: Vec<_> = self
            .module_caches
            .borrow()
            .values()
            .flat_map(|cache| cache.lock().cached_module_datum().into_iter())
            .collect();
        flatten.sort_unstable_by_key(|(key, ..)| *key);
        flatten.into_iter().map(|(_, address, item)| (address, item)).collect()
    }

    pub fn root(&self) -> H256 {
        self.root
    }
}

// TODO: cloning for `State` shouldn't be possible in general; Remove this and use
// checkpoints where possible.
impl Clone for TopLevelState {
    fn clone(&self) -> TopLevelState {
        let module_caches = self
            .module_caches
            .borrow()
            .iter()
            .map(|(id, cache)| (*id, Arc::new(Mutex::new(cache.lock().clone()))))
            .collect();
        TopLevelState {
            db: Arc::new(RwLock::new(self.db.read().clone(&self.root))),
            root: self.root,
            id_of_checkpoints: self.id_of_checkpoints.clone(),
            top_cache: self.top_cache.clone(),
            module_caches: RefCell::new(module_caches),
        }
    }
}

impl TopState for TopLevelState {
    fn create_module(&mut self) -> StateResult<()> {
        let storage_id = {
            let mut metadata = self.get_metadata_mut()?;
            metadata.add_module()
        };
        self.create_module_level_state(storage_id)?;

        Ok(())
    }

    fn set_module_root(&mut self, storage_id: StorageId, new_root: H256) -> StateResult<()> {
        let mut module = self.get_module_mut(storage_id)?;
        module.set_root(new_root);
        Ok(())
    }

    fn increase_term_id(&mut self, last_term_finished_block_num: u64) -> StateResult<()> {
        let mut metadata = self.get_metadata_mut()?;
        metadata.increase_term_id(last_term_finished_block_num);
        Ok(())
    }

    fn update_action_data(&mut self, key: &H256, data: Bytes) -> StateResult<()> {
        let mut action_data = self.get_action_data_mut(key)?;
        *action_data = data.into();
        Ok(())
    }

    fn remove_action_data(&mut self, key: &H256) {
        self.top_cache.remove_action_data(key)
    }

    fn update_params(&mut self, metadata_seq: u64, params: CommonParams) -> StateResult<()> {
        let mut metadata = self.get_metadata_mut()?;
        if metadata.seq() != metadata_seq {
            return Err(RuntimeError::InvalidSeq(Mismatch {
                found: metadata_seq,
                expected: metadata.seq(),
            })
            .into())
        }

        metadata.set_params(params);
        metadata.increase_seq();
        Ok(())
    }

    fn update_consensus_params(&mut self, consensus_params: ConsensusParams) -> StateResult<()> {
        let mut metadata = self.get_metadata_mut()?;
        metadata.set_consensus_params(consensus_params);
        Ok(())
    }
}

#[cfg(test)]
mod test_module_states {
    use super::*;
    use crate::tests::helpers::get_temp_state;

    #[test]
    fn create_module_states() {
        let mut top_level_state = get_temp_state();
        // create module with StorageId = 0
        let module_count = 10;
        let storage_id_range = 0..module_count;
        storage_id_range.clone().for_each(|_| {
            top_level_state.create_module().unwrap();
        });
        storage_id_range.for_each(|storage_id| {
            top_level_state.module_state_mut(storage_id).unwrap();
            assert!(top_level_state.module_state(storage_id).unwrap().is_some());
        })
    }

    #[test]
    fn commit() {
        let mut top_level_state = get_temp_state();
        let storage_id_0: StorageId = 0;
        let storage_id_1: StorageId = 1;
        top_level_state.create_module().unwrap();
        top_level_state.create_module().unwrap();
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer")
                ]
            });
            let state_with_id_1 = top_level_state.module_state_mut(storage_id_1).unwrap();
            module_level!(state_with_id_1, {
                set: [
                    (key: "charlie" => datum_str: "Charlie is a physicist"),
                    (key: "dave" => datum_str: "Dave is a singer")
                ]
            });
        }
        let (db, root) = top_level_state.commit_and_clone_db().unwrap();
        let mut top_level_state = TopLevelState::from_existing(db, root).unwrap();
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                check: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer"),
                    (key: "charlie" => None),
                    (key: "dave" => None)
                ]
            });
            let state_with_id_1 = top_level_state.module_state_mut(storage_id_1).unwrap();
            module_level!(state_with_id_1, {
                check: [
                    (key: "alice" => None),
                    (key: "bob" => None),
                    (key: "charlie" => datum_str: "Charlie is a physicist"),
                    (key: "dave" => datum_str: "Dave is a singer")
                ]
            });
        }
    }

    #[test]
    fn without_commit() {
        let mut top_level_state = get_temp_state();
        let storage_id_0: StorageId = 0;
        top_level_state.create_module().unwrap();
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer")
                ]
            });
        }
        let (state_db, root) = top_level_state.commit_and_clone_db().unwrap();
        let mut top_level_state = TopLevelState::from_existing(state_db.clone(&root), root).unwrap();
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                check: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer"),
                    (key: "charlie" => None),
                ]
            });
        }
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice became a software engineer"),
                    (key: "bob" => datum_str: "Bob became a doctor"),
                    (key: "charlie" => datum_str: "Charlie is a nurse"),
                ]
            });
        }
        let mut top_level_state = TopLevelState::from_existing(state_db.clone(&root), root).unwrap();
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                check: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer"),
                    (key: "charlie" => None)
                ]
            });
        }
    }

    #[test]
    fn checkpoint_and_revert() {
        let mut top_level_state = get_temp_state();
        let storage_id_0: StorageId = 0;
        top_level_state.create_module().unwrap();
        // state1
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer")
                ]
            });
        }
        let checkpoint1 = 1;
        StateWithCheckpoint::create_checkpoint(&mut top_level_state, checkpoint1);
        // state2
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice became a software engineer"),
                    (key: "bob" => datum_str: "Bob became a doctor")
                ]
            });
        }
        let checkpoint2 = 2;
        StateWithCheckpoint::create_checkpoint(&mut top_level_state, checkpoint2);
        // state3
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "charlie" => datum_str: "Charlie is a nurse")
                ]
            });
        }

        //state3
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                check: [
                    (key: "alice" => datum_str: "Alice became a software engineer"),
                    (key: "bob" => datum_str: "Bob became a doctor"),
                    (key: "charlie" => datum_str: "Charlie is a nurse")
                ]
            });
        }

        StateWithCheckpoint::revert_to_checkpoint(&mut top_level_state, checkpoint2);
        //state2
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                check: [
                    (key: "alice" => datum_str: "Alice became a software engineer"),
                    (key: "bob" => datum_str: "Bob became a doctor"),
                    (key: "charlie" => None)
                ]
            });
        }

        StateWithCheckpoint::revert_to_checkpoint(&mut top_level_state, checkpoint1);
        //state1
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                check: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer"),
                    (key: "charlie" => None)
                ]
            });
        }
    }

    #[test]
    fn checkpoint_discard_and_revert() {
        let mut top_level_state = get_temp_state();
        let storage_id_0: StorageId = 0;
        top_level_state.create_module().unwrap();
        // state1
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer")
                ]
            });
        }
        let checkpoint1 = 1;
        StateWithCheckpoint::create_checkpoint(&mut top_level_state, checkpoint1);
        // state2
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice became a software engineer"),
                    (key: "bob" => datum_str: "Bob became a doctor")
                ]
            });
        }
        let checkpoint2 = 2;
        StateWithCheckpoint::create_checkpoint(&mut top_level_state, checkpoint2);
        // state3
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "charlie" => datum_str: "Charlie is a nurse")
                ]
            });
        }

        //state3
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                check: [
                    (key: "alice" => datum_str: "Alice became a software engineer"),
                    (key: "bob" => datum_str: "Bob became a doctor"),
                    (key: "charlie" => datum_str: "Charlie is a nurse")
                ]
            });
        }

        StateWithCheckpoint::discard_checkpoint(&mut top_level_state, checkpoint2);
        StateWithCheckpoint::revert_to_checkpoint(&mut top_level_state, checkpoint1);
        //state1
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                check: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer"),
                    (key: "charlie" => None)
                ]
            });
        }
    }

    #[test]
    #[should_panic]
    fn commit_and_restore_do_not_preserve_checkpoints() {
        let mut top_level_state = get_temp_state();
        let storage_id_0: StorageId = 0;
        top_level_state.create_module().unwrap();
        // state1
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice is a doctor"),
                    (key: "bob" => datum_str: "Bob is a software engineer")
                ]
            });
        }
        let checkpoint1 = 1;
        StateWithCheckpoint::create_checkpoint(&mut top_level_state, checkpoint1);
        // state2
        {
            let state_with_id_0 = top_level_state.module_state_mut(storage_id_0).unwrap();
            module_level!(state_with_id_0, {
                set: [
                    (key: "alice" => datum_str: "Alice became a software engineer"),
                    (key: "bob" => datum_str: "Bob became a doctor")
                ]
            });
        }
        let (db, root) = top_level_state.commit_and_clone_db().unwrap();
        let mut top_level_state = TopLevelState::from_existing(db, root).unwrap();

        StateWithCheckpoint::revert_to_checkpoint(&mut top_level_state, checkpoint1);
    }
}
