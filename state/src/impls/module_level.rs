// Copyright 2020 Kodebox, Inc.
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

use crate::cache::ModuleCache;
use crate::checkpoint::{CheckpointId, StateWithCheckpoint};
use crate::traits::ModuleStateView;
use crate::{ModuleDatum, ModuleDatumAddress, StateDB, StateResult};
use ccrypto::BLAKE_NULL_RLP;
use cdb::AsHashDB;
use coordinator::context::SubStorageAccess;
use ctypes::StorageId;
use merkle_trie::{Result as TrieResult, TrieError, TrieFactory};
use parking_lot::{Mutex, RwLock};
use primitives::H256;
use remote_trait_object::Service;
use std::sync::Arc;

pub struct ModuleLevelState {
    db: Arc<RwLock<StateDB>>,
    root: H256,
    cache: Arc<Mutex<ModuleCache>>,
    id_of_checkpoints: Vec<CheckpointId>,
    storage_id: StorageId,
}

impl ModuleLevelState {
    /// Creates new state with empty state root
    pub fn try_new(
        storage_id: StorageId,
        db: Arc<RwLock<StateDB>>,
        cache: Arc<Mutex<ModuleCache>>,
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
        db: Arc<RwLock<StateDB>>,
        root: H256,
        cache: Arc<Mutex<ModuleCache>>,
    ) -> TrieResult<Self> {
        if !db.read().as_hashdb().contains(&root) {
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

    pub fn set_datum(&self, key: &dyn AsRef<[u8]>, datum: Vec<u8>) -> StateResult<()> {
        let db = self.db.write();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let cache = self.cache.lock();
        let mut datum_mut = cache.module_datum_mut(&ModuleDatumAddress::new(key, self.storage_id), &trie)?;
        *datum_mut = ModuleDatum::new(datum);
        Ok(())
    }

    pub fn remove_key(&self, key: &dyn AsRef<[u8]>) {
        self.cache.lock().remove_module_datum(&ModuleDatumAddress::new(key, self.storage_id))
    }
}

impl ModuleStateView for ModuleLevelState {
    fn get_datum(&self, key: &dyn AsRef<[u8]>) -> Result<Option<ModuleDatum>, TrieError> {
        let db = self.db.read();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.lock().module_datum(&ModuleDatumAddress::new(key, self.storage_id), &trie)
    }

    fn has_key(&self, key: &dyn AsRef<[u8]>) -> TrieResult<bool> {
        let db = self.db.read();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.lock().has(&ModuleDatumAddress::new(key, self.storage_id), &trie)
    }
}

impl StateWithCheckpoint for ModuleLevelState {
    fn create_checkpoint(&mut self, id: CheckpointId) {
        ctrace!(STATE, "Checkpoint({}) for module({}) is created", id, self.storage_id);
        self.id_of_checkpoints.push(id);
        self.cache.lock().checkpoint();
    }

    fn discard_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for module({}) is discarded", id, self.storage_id);
        self.cache.lock().discard_checkpoint();
    }

    fn revert_to_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for module({}) is reverted", id, self.storage_id);
        self.cache.lock().revert_to_checkpoint();
    }
}

macro_rules! panic_at {
    ($method: literal, $e: expr) => {
        panic!("SubStorageAccess {} method failed with {}", $method, $e);
    };
}

impl Service for ModuleLevelState {}

impl SubStorageAccess for ModuleLevelState {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        match self.get_datum(&key) {
            Ok(datum) => datum.map(|datum| datum.content()),
            Err(e) => panic_at!("get", e),
        }
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        if let Err(e) = self.set_datum(&key, value) {
            panic_at!("set", e)
        }
    }

    fn has(&self, key: &[u8]) -> bool {
        match self.has_key(&key) {
            Ok(result) => result,
            Err(e) => panic_at!("has", e),
        }
    }

    fn remove(&mut self, key: &[u8]) {
        self.remove_key(&key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers::get_temp_state_db;

    const STORAGE_ID: StorageId = 4;
    const CHECKPOINT_ID: usize = 777;

    fn get_temp_module_state(
        state_db: Arc<RwLock<StateDB>>,
        storage_id: StorageId,
        cache: Arc<RwLock<ModuleCache>>,
    ) -> ModuleLevelState {
        ModuleLevelState::try_new(storage_id, state_db, cache).unwrap()
    }

    #[test]
    fn set_module_datum() {
        let state_db = Arc::new(RwLock::new(get_temp_state_db()));
        let module_cache = Arc::new(RwLock::new(ModuleCache::default()));
        let state = get_temp_module_state(state_db, STORAGE_ID, module_cache);

        let key = "datum key";
        let datum = "module_datum";

        module_level!(state, {
            set: [(key: key => datum_str: datum)],
            check: [(key: key => datum_str: datum)],
        });
    }

    #[test]
    fn checkpoint_and_revert() {
        let state_db = Arc::new(RwLock::new(get_temp_state_db()));
        let module_cache = Arc::new(RwLock::new(ModuleCache::default()));
        let mut state = get_temp_module_state(state_db, STORAGE_ID, module_cache);

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
        let state_db = Arc::new(RwLock::new(get_temp_state_db()));
        let module_cache = Arc::new(RwLock::new(ModuleCache::default()));
        let mut state = get_temp_module_state(state_db, STORAGE_ID, module_cache);

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
        let state_db = Arc::new(RwLock::new(get_temp_state_db()));
        let module_cache = Arc::new(RwLock::new(ModuleCache::default()));
        let mut state = get_temp_module_state(state_db, STORAGE_ID, module_cache);

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
