// Copyright 2020 Kodebox, Inc.
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

use crate::cache::ModuleCache;
use crate::checkpoint::{CheckpointId, StateWithCheckpoint};
use crate::traits::ModuleStateView;
use crate::{ModuleDatum, ModuleDatumAddress, StateDB, StateResult};
use ccrypto::BLAKE_NULL_RLP;
use cdb::AsHashDB;
use ctypes::StorageId;
use merkle_trie::{Result as TrieResult, TrieError, TrieFactory};
use primitives::H256;
use std::cell::RefCell;

pub struct ModuleLevelState<'db> {
    db: &'db mut RefCell<StateDB>,
    root: H256,
    cache: &'db mut ModuleCache,
    id_of_checkpoints: Vec<CheckpointId>,
    storage_id: StorageId,
}

impl<'db> ModuleLevelState<'db> {
    /// Creates new state with empty state root
    pub fn try_new(
        storage_id: StorageId,
        db: &'db mut RefCell<StateDB>,
        cache: &'db mut ModuleCache,
    ) -> StateResult<Self> {
        let root = BLAKE_NULL_RLP;
        Ok(Self {
            db,
            root,
            cache,
            id_of_checkpoints: Default::default(),
            storage_id,
        })
    }

    /// Creates new state with existing state root
    pub fn from_existing(
        storage_id: StorageId,
        db: &'db mut RefCell<StateDB>,
        root: H256,
        cache: &'db mut ModuleCache,
    ) -> TrieResult<Self> {
        if !db.borrow().as_hashdb().contains(&root) {
            return Err(TrieError::InvalidStateRoot(root))
        }

        Ok(Self {
            db,
            root,
            cache,
            id_of_checkpoints: Default::default(),
            storage_id,
        })
    }

    /// Creates immutable module state
    pub fn read_only(
        storage_id: StorageId,
        db: &RefCell<StateDB>,
        root: H256,
        cache: ModuleCache,
    ) -> TrieResult<ReadOnlyModuleLevelState<'_>> {
        if !db.borrow().as_hashdb().contains(&root) {
            return Err(TrieError::InvalidStateRoot(root))
        }

        Ok(ReadOnlyModuleLevelState {
            db,
            root,
            cache,
            storage_id,
        })
    }

    pub fn set_datum(&self, key: &dyn AsRef<[u8]>, datum: Vec<u8>) -> StateResult<()> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let mut datum_mut = self.cache.module_datum_mut(&ModuleDatumAddress::new(key, self.storage_id), &trie)?;
        *datum_mut = ModuleDatum::new(datum);
        Ok(())
    }

    pub fn remove_key(&self, key: &dyn AsRef<[u8]>) {
        self.cache.remove_module_datum(&ModuleDatumAddress::new(key, self.storage_id))
    }
}

impl<'db> ModuleStateView for ModuleLevelState<'db> {
    fn get_datum(&self, key: &dyn AsRef<[u8]>) -> Result<Option<ModuleDatum>, TrieError> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.module_datum(&ModuleDatumAddress::new(key, self.storage_id), &trie)
    }

    fn has_key(&self, key: &dyn AsRef<[u8]>) -> TrieResult<bool> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.has(&ModuleDatumAddress::new(key, self.storage_id), &trie)
    }
}

impl<'db> StateWithCheckpoint for ModuleLevelState<'db> {
    fn create_checkpoint(&mut self, id: CheckpointId) {
        ctrace!(STATE, "Checkpoint({}) for module({}) is created", id, self.storage_id);
        self.id_of_checkpoints.push(id);
        self.cache.checkpoint();
    }

    fn discard_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for module({}) is discarded", id, self.storage_id);
        self.cache.discard_checkpoint();
    }

    fn revert_to_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for module({}) is reverted", id, self.storage_id);
        self.cache.revert_to_checkpoint();
    }
}

pub struct ReadOnlyModuleLevelState<'db> {
    db: &'db RefCell<StateDB>,
    root: H256,
    cache: ModuleCache,
    storage_id: StorageId,
}

impl<'db> ModuleStateView for ReadOnlyModuleLevelState<'db> {
    fn get_datum(&self, key: &dyn AsRef<[u8]>) -> Result<Option<ModuleDatum>, TrieError> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.module_datum(&ModuleDatumAddress::new(key, self.storage_id), &trie)
    }

    fn has_key(&self, key: &dyn AsRef<[u8]>) -> TrieResult<bool> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.has(&ModuleDatumAddress::new(key, self.storage_id), &trie)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::get_temp_state_db;

    const STORAGE_ID: StorageId = 4;
    const CHECKPOINT_ID: usize = 777;

    fn get_temp_module_state<'d>(
        state_db: &'d mut RefCell<StateDB>,
        storage_id: StorageId,
        cache: &'d mut ModuleCache,
    ) -> ModuleLevelState<'d> {
        ModuleLevelState::try_new(storage_id, state_db, cache).unwrap()
    }

    #[test]
    fn set_module_datum() {
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut module_cache = ModuleCache::default();
        let state = get_temp_module_state(&mut state_db, STORAGE_ID, &mut module_cache);

        let key = "datum key";
        let datum = "module_datum";

        module_level!(state, {
            set: [(key: key => datum_str: datum)],
            check: [(key: key => datum_str: datum)],
        });
    }

    #[test]
    fn checkpoint_and_revert() {
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut module_cache = ModuleCache::default();
        let mut state = get_temp_module_state(&mut state_db, STORAGE_ID, &mut module_cache);

        // state 1
        let key1 = "datum key 1";
        let datum = "module datum";
        module_level!(state, {
            set: [(key: key1 => datum_str: datum)],
            check: [(key: key1 => datum_str: datum)]
        });
        state.create_checkpoint(CHECKPOINT_ID);

        // state 2
        let modified_datum = "A modified module datum";
        let key2 = "datum key 2";
        let new_datum = "A new module datum";
        module_level!(state, {
            set: [
                (key: key1 => datum_str: modified_datum),
                (key: key2 => datum_str: new_datum)
            ],
            check: [
                (key: key1 => datum_str: modified_datum),
                (key: key2 => datum_str: new_datum)
            ],
        });

        // state 1
        state.revert_to_checkpoint(CHECKPOINT_ID);
        module_level!(state, {
            check: [
                (key: key1 => datum_str: datum),
                (key: key2 => None)
            ]
        });
    }

    #[test]
    fn checkpoint_discard_and_revert() {
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut module_cache = ModuleCache::default();
        let mut state = get_temp_module_state(&mut state_db, STORAGE_ID, &mut module_cache);

        // state 1
        let key = "datum key";
        let datum = "module datum";
        module_level!(state, {
            set: [(key: key => datum_str: datum)],
            check: [(key: key => datum_str: datum)]
        });
        state.create_checkpoint(CHECKPOINT_ID);

        // state 2
        let another_key = "another datum key";
        let modified_datum_1 = "A modified module datum 1";
        let another_datum = "another module datum";
        module_level!(state, {
            set: [
                (key: key => datum_str: modified_datum_1),
                (key: another_key => datum_str: another_datum)
            ],
            check: [
                (key: key => datum_str: modified_datum_1),
                (key: another_key => datum_str: another_datum)
            ],
        });
        state.create_checkpoint(CHECKPOINT_ID);

        // state 3
        let modified_datum_2 = "A modified module datum 2";
        module_level!(state, {
            set: [(key: key => datum_str: modified_datum_2)],
            check: [(key: key => datum_str: modified_datum_2)],
        });
        state.create_checkpoint(CHECKPOINT_ID);

        // state 3 checkpoint merged into state 2
        state.discard_checkpoint(CHECKPOINT_ID);

        // Revert to the state 2
        state.revert_to_checkpoint(CHECKPOINT_ID);
        module_level!(state, {
            check: [
                (key: key => datum_str: modified_datum_1),
                (key: another_key => datum_str: another_datum)
            ]
        });

        // Revert to the state 1
        state.revert_to_checkpoint(CHECKPOINT_ID);
        module_level!(state, {
            check: [
                (key: key => datum_str: datum),
                (key: another_key => None)
            ]
        });
    }

    #[test]
    fn checkpoint_and_revert_with_remove() {
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut module_cache = ModuleCache::default();
        let mut state = get_temp_module_state(&mut state_db, STORAGE_ID, &mut module_cache);

        // state 1
        let key1 = "datum key1";
        let datum1 = "module datum1";
        let key2 = "datum key2";
        let datum2 = "module datum2";
        module_level!(state, {
            set: [
                (key: key1 => datum_str: datum1),
                (key: key2 => datum_str: datum2)
            ]
        });
        state.create_checkpoint(CHECKPOINT_ID);

        // state 2: key2 removed
        state.remove_key(&key2);
        state.create_checkpoint(CHECKPOINT_ID);
        module_level!(state, {
            check: [(key: key2 => None)]
        });

        // state 3: key1 removed
        state.remove_key(&key1);
        module_level!(state, {
            check: [(key: key1 => None)]
        });

        // state 4: key1 revived
        state.revert_to_checkpoint(CHECKPOINT_ID);
        module_level!(state, {
            check: [
                (key: key1 => Some),
                (key: key2 => None)
            ]
        });

        // state 5: key2 revived
        state.revert_to_checkpoint(CHECKPOINT_ID);
        module_level!(state, {
            check: [
                (key: key1 => Some),
                (key: key2 => Some)
            ]
        });
    }
}
