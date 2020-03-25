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
    Account, ActionData, FindActionHandler, Metadata, MetadataAddress, Module, ModuleAddress, ModuleLevelState,
    StateDB, StateResult,
};
use ccrypto::BLAKE_NULL_RLP;
use cdb::{AsHashDB, DatabaseError};
use ckey::{public_to_address, Address, Ed25519Public as Public, NetworkId};
use coordinator::context::{Key as DbCxtKey, SubStorageAccess, Value as DbCxtValue};
use ctypes::errors::RuntimeError;
use ctypes::transaction::{Action, Transaction};
use ctypes::util::unexpected::Mismatch;
use ctypes::{BlockNumber, CommonParams, StorageId, TxHash};
use kvdb::DBTransaction;
use merkle_trie::{Result as TrieResult, TrieError, TrieFactory};
use primitives::{Bytes, H256};
use std::cell::{RefCell, RefMut};
use std::collections::HashMap;

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
    db: RefCell<StateDB>,
    root: H256,

    top_cache: TopCache,
    module_caches: HashMap<StorageId, ModuleCache>,
    id_of_checkpoints: Vec<CheckpointId>,
}

impl TopStateView for TopLevelState {
    /// Check caches for required data
    /// First searches for account in the local, then the shared cache.
    /// Populates local cache if nothing found.
    fn account(&self, a: &Address) -> TrieResult<Option<Account>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.account(&a, &trie)
    }

    fn metadata(&self) -> TrieResult<Option<Metadata>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let address = MetadataAddress::new();
        self.top_cache.metadata(&address, &trie)
    }

    fn module(&self, storage_id: StorageId) -> TrieResult<Option<Module>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let module_address = ModuleAddress::new(storage_id);
        self.top_cache.module(&module_address, &trie)
    }

    fn module_state<'db>(&'db self, storage_id: StorageId) -> TrieResult<Option<Box<dyn ModuleStateView + 'db>>> {
        match self.module_root(storage_id)? {
            // FIXME: Find a way to use stored cache.
            Some(module_root) => {
                let module_cache = self.module_caches.get(&storage_id).cloned().unwrap_or_default();
                Ok(Some(Box::new(ModuleLevelState::read_only(storage_id, &self.db, module_root, module_cache)?)))
            }
            None => Ok(None),
        }
    }

    fn action_data(&self, key: &H256) -> TrieResult<Option<ActionData>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        Ok(self.top_cache.action_data(key, &trie)?.map(Into::into))
    }
}

macro_rules! panic_at {
    ($method: literal, $e: expr) => {
        panic!("SubStorageAccess {} method failed with {}", $method, $e);
    };
}

impl SubStorageAccess for TopLevelState {
    fn get(&self, storage_id: StorageId, key: &DbCxtKey) -> Option<DbCxtValue> {
        match self.module_state(storage_id) {
            Ok(state) => match state?.get_datum(key) {
                Ok(datum) => datum.map(|datum| datum.content()),
                Err(e) => panic_at!("get", e),
            },
            Err(e) => panic_at!("get", e),
        }
    }

    fn set(&mut self, storage_id: StorageId, key: &DbCxtKey, value: DbCxtValue) {
        match self.module_state_mut(storage_id) {
            Ok(state) => {
                if let Err(e) = state.set_datum(key, value) {
                    panic_at!("set", e)
                }
            }
            Err(e) => panic_at!("set", e),
        }
    }

    fn has(&self, storage_id: StorageId, key: &DbCxtKey) -> bool {
        match self.module_state(storage_id) {
            Ok(state) => state
                .map(|state| match state.has_key(key) {
                    Ok(result) => result,
                    Err(e) => panic_at!("has", e),
                })
                .unwrap_or(false),
            Err(e) => panic_at!("has", e),
        }
    }

    fn remove(&mut self, storage_id: StorageId, key: &DbCxtKey) {
        match self.module_state_mut(storage_id) {
            Ok(state) => state.remove_key(key),
            Err(e) => panic_at!("remove", e),
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
            .iter()
            .map(|(&storage_id, _)| {
                if let Some(module_root) = self.module_root(storage_id)? {
                    Ok(Some((storage_id, module_root)))
                } else {
                    Ok(None)
                }
            })
            .collect::<StateResult<Vec<_>>>()?;
        for (storage_id, mut module_root) in module_changes.into_iter().filter_map(|result| result) {
            {
                let mut db = self.db.borrow_mut();
                let mut trie = TrieFactory::from_existing(db.as_hashdb_mut(), &mut module_root)?;

                let module_cache = self.module_caches.get_mut(&storage_id).expect("Module must exist");

                module_cache.commit(&mut trie)?;
            }
            self.set_module_root(storage_id, module_root)?;
        }
        {
            let mut db = self.db.borrow_mut();
            let mut trie = TrieFactory::from_existing(db.as_hashdb_mut(), &mut self.root)?;
            self.top_cache.commit(&mut trie)?;
        }
        Ok(self.root)
    }

    fn commit_and_into_db(mut self) -> StateResult<(StateDB, H256)> {
        let root = self.commit()?;
        Ok((self.db.into_inner(), root))
    }
}

const TOP_CHECKPOINT: CheckpointId = 777;
const FEE_CHECKPOINT: CheckpointId = 123;
const ACTION_CHECKPOINT: CheckpointId = 130;

impl StateWithCheckpoint for TopLevelState {
    fn create_checkpoint(&mut self, id: CheckpointId) {
        ctrace!(STATE, "Checkpoint({}) for top level is created", id);
        self.id_of_checkpoints.push(id);
        self.top_cache.checkpoint();

        self.module_caches.iter_mut().for_each(|(_, cache)| cache.checkpoint())
    }

    fn discard_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is discarded", id);
        self.top_cache.discard_checkpoint();

        self.module_caches.iter_mut().for_each(|(_, cache)| cache.discard_checkpoint())
    }

    fn revert_to_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is reverted", id);
        self.top_cache.revert_to_checkpoint();

        self.module_caches.iter_mut().for_each(|(_, cache)| cache.revert_to_checkpoint())
    }
}

impl TopLevelState {
    /// Creates new state with existing state root
    pub fn from_existing(db: StateDB, root: H256) -> Result<Self, TrieError> {
        if !db.as_hashdb().contains(&root) {
            return Err(TrieError::InvalidStateRoot(root))
        }

        let top_cache = db.top_cache();
        let module_caches = db.module_caches();

        let state = TopLevelState {
            db: RefCell::new(db),
            root,
            top_cache,
            module_caches,
            id_of_checkpoints: Default::default(),
        };

        Ok(state)
    }

    /// Execute a given tranasction, charging tranasction fee.
    /// This will change the state accordingly.
    pub fn apply<C: FindActionHandler>(
        &mut self,
        tx: &Transaction,
        signed_hash: &TxHash,
        sender_public: &Public,
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
        current_block_timestamp: u64,
    ) -> StateResult<()> {
        StateWithCheckpoint::create_checkpoint(self, FEE_CHECKPOINT);
        let result = self.apply_internal(
            tx,
            signed_hash,
            sender_public,
            client,
            parent_block_number,
            parent_block_timestamp,
            current_block_timestamp,
        );
        match result {
            Ok(()) => {
                StateWithCheckpoint::discard_checkpoint(self, FEE_CHECKPOINT);
            }
            Err(_) => {
                self.revert_to_checkpoint(FEE_CHECKPOINT);
            }
        }
        result
    }

    fn apply_internal<C: FindActionHandler>(
        &mut self,
        tx: &Transaction,
        signed_hash: &TxHash,
        sender_public: &Public,
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
        current_block_timestamp: u64,
    ) -> StateResult<()> {
        let sender_address = public_to_address(sender_public);
        let seq = self.seq(&sender_address)?;

        if tx.seq != seq {
            return Err(RuntimeError::InvalidSeq(Mismatch {
                expected: seq,
                found: tx.seq,
            })
            .into())
        }

        let fee = tx.fee;

        self.inc_seq(&sender_address)?;
        self.sub_balance(&sender_address, fee)?;

        // The failed transaction also must pay the fee and increase seq.
        StateWithCheckpoint::create_checkpoint(self, ACTION_CHECKPOINT);
        let result = self.apply_action(
            &tx.action,
            tx.network_id,
            tx.hash(),
            signed_hash,
            &sender_address,
            sender_public,
            client,
            parent_block_number,
            parent_block_timestamp,
            current_block_timestamp,
        );
        match &result {
            Ok(()) => {
                StateWithCheckpoint::discard_checkpoint(self, ACTION_CHECKPOINT);
            }
            Err(_) => {
                self.revert_to_checkpoint(ACTION_CHECKPOINT);
            }
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_action<C: FindActionHandler>(
        &mut self,
        action: &Action,
        _network_id: NetworkId,
        _tx_hash: TxHash,
        _signed_hash: &TxHash,
        sender_address: &Address,
        sender_public: &Public,
        client: &C,
        _parent_block_number: BlockNumber,
        _parent_block_timestamp: u64,
        _current_block_timestamp: u64,
    ) -> StateResult<()> {
        match action {
            Action::Pay {
                receiver,
                quantity,
            } => {
                self.transfer_balance(sender_address, receiver, *quantity)?;
                Ok(())
            }
            Action::Custom {
                handler_id,
                bytes,
            } => {
                let handler = client.find_action_handler_for(*handler_id).expect("Unknown custom parsel applied!");
                handler.execute(bytes, self, sender_address, sender_public)?;
                Ok(())
            }
        }
    }

    fn create_module_level_state(&mut self, storage_id: StorageId) -> StateResult<()> {
        const DEFAULT_MODULE_ROOT: H256 = BLAKE_NULL_RLP;
        {
            let module_cache = self.module_caches.entry(storage_id).or_default();
            ModuleLevelState::from_existing(storage_id, &mut self.db, DEFAULT_MODULE_ROOT, module_cache)?;
        }
        ctrace!(STATE, "module storage({}) created", storage_id);
        self.set_module_root(storage_id, DEFAULT_MODULE_ROOT)?;
        Ok(())
    }

    fn module_state_mut(&mut self, storage_id: StorageId) -> StateResult<ModuleLevelState> {
        let module_root = self.module_root(storage_id)?.ok_or_else(|| RuntimeError::InvalidStorageId(storage_id))?;
        let module_cache = self.module_caches.get_mut(&storage_id).expect("storage id is verified");
        Ok(ModuleLevelState::from_existing(storage_id, &mut self.db, module_root, module_cache)?)
    }

    fn get_account_mut(&self, a: &Address) -> TrieResult<RefMut<'_, Account>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.account_mut(&a, &trie)
    }

    fn get_metadata_mut(&self) -> TrieResult<RefMut<'_, Metadata>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let address = MetadataAddress::new();
        self.top_cache.metadata_mut(&address, &trie)
    }

    fn get_module_mut(&self, storage_id: StorageId) -> TrieResult<RefMut<'_, Module>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let address = ModuleAddress::new(storage_id);
        self.top_cache.module_mut(&address, &trie)
    }

    fn get_action_data_mut(&self, key: &H256) -> TrieResult<RefMut<'_, ActionData>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.action_data_mut(key, &trie)
    }

    pub fn journal_under(&self, batch: &mut DBTransaction, now: u64) -> Result<u32, DatabaseError> {
        self.db.borrow_mut().journal_under(batch, now, self.root)
    }

    pub fn top_cache(&self) -> &TopCache {
        &self.top_cache
    }

    pub fn module_caches(&self) -> &HashMap<StorageId, ModuleCache> {
        &self.module_caches
    }

    pub fn root(&self) -> H256 {
        self.root
    }

    #[cfg(test)]
    fn set_balance(&mut self, a: &Address, balance: u64) -> TrieResult<()> {
        self.get_account_mut(a)?.set_balance(balance);
        Ok(())
    }
}

// TODO: cloning for `State` shouldn't be possible in general; Remove this and use
// checkpoints where possible.
impl Clone for TopLevelState {
    fn clone(&self) -> TopLevelState {
        TopLevelState {
            db: RefCell::new(self.db.borrow().clone(&self.root)),
            root: self.root,
            id_of_checkpoints: self.id_of_checkpoints.clone(),
            top_cache: self.top_cache.clone(),
            module_caches: self.module_caches.clone(),
        }
    }
}

impl TopState for TopLevelState {
    fn kill_account(&mut self, account: &Address) {
        self.top_cache.remove_account(account);
    }

    fn add_balance(&mut self, a: &Address, incr: u64) -> TrieResult<()> {
        ctrace!(STATE, "add_balance({}, {}): {}", a, incr, self.balance(a)?);
        if incr != 0 {
            self.get_account_mut(a)?.add_balance(incr);
        }
        Ok(())
    }

    fn sub_balance(&mut self, a: &Address, decr: u64) -> StateResult<()> {
        ctrace!(STATE, "sub_balance({}, {}): {}", a, decr, self.balance(a)?);
        if decr == 0 {
            return Ok(())
        }
        let balance = self.balance(a)?;
        if balance < decr {
            return Err(RuntimeError::InsufficientBalance {
                address: *a,
                cost: decr,
                balance,
            }
            .into())
        }
        self.get_account_mut(a)?.sub_balance(decr);
        Ok(())
    }

    fn transfer_balance(&mut self, from: &Address, to: &Address, by: u64) -> StateResult<()> {
        self.sub_balance(from, by)?;
        self.add_balance(to, by)?;
        Ok(())
    }

    fn inc_seq(&mut self, a: &Address) -> TrieResult<()> {
        self.get_account_mut(a)?.inc_seq();
        Ok(())
    }

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

    fn update_term_params(&mut self) -> StateResult<()> {
        let mut metadata = self.get_metadata_mut()?;
        metadata.update_term_params();
        Ok(())
    }
}

#[cfg(test)]
mod tests_state {
    use std::sync::Arc;

    use cdb::{new_journaldb, Algorithm};

    use super::*;
    use crate::tests::helpers::{
        empty_top_state, empty_top_state_with_metadata, get_memory_db, get_temp_state, get_temp_state_db,
    };

    #[test]
    fn work_when_cloned() {
        let a = Address::default();

        let mut state = {
            let mut state = get_temp_state();
            assert_eq!(Ok(false), state.account_exists(&a));
            assert_eq!(Ok(()), state.inc_seq(&a));
            assert_eq!(Ok(1), state.seq(&a));
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            state
        };
        assert_eq!(Ok(1), state.seq(&a));
        assert_eq!(Ok(()), state.inc_seq(&a));
        assert_eq!(Ok(2), state.seq(&a));
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        assert_eq!(Ok(2), state.seq(&a));
    }

    #[test]
    fn work_when_cloned_even_not_committed() {
        let a = Address::default();

        let mut state = {
            let mut state = get_temp_state();
            assert_eq!(Ok(false), state.account_exists(&a));
            assert_eq!(Ok(()), state.inc_seq(&a));
            assert_eq!(Ok(1), state.seq(&a));
            state
        };
        assert_eq!(Ok(1), state.seq(&a));
        assert_eq!(Ok(()), state.inc_seq(&a));
        assert_eq!(Ok(2), state.seq(&a));
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        assert_eq!(Ok(2), state.seq(&a));
    }

    #[test]
    fn state_is_not_synchronized_when_cloned() {
        let a = Address::random();

        let original_state = get_temp_state();

        assert_eq!(Ok(false), original_state.account_exists(&a));

        let mut cloned_state = original_state.clone();

        assert_eq!(Ok(()), cloned_state.inc_seq(&a));
        let root = cloned_state.commit();
        assert!(root.is_ok(), "{:?}", root);

        assert_ne!(original_state.seq(&a), cloned_state.seq(&a));
    }

    #[test]
    fn get_from_database() {
        let memory_db = get_memory_db();
        let jorunal = new_journaldb(Arc::clone(&memory_db), Algorithm::Archive, Some(0));
        let db = StateDB::new(jorunal.boxed_clone());
        let a = Address::default();
        let root = {
            let mut state = empty_top_state_with_metadata(db.clone(&H256::zero()), CommonParams::default_for_test());
            assert_eq!(Ok(()), state.inc_seq(&a));
            assert_eq!(Ok(()), state.add_balance(&a, 100));
            assert_eq!(Ok(100), state.balance(&a));
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(100), state.balance(&a));

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(4, records.unwrap());
            memory_db.write_buffered(transaction);

            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(100), state.balance(&a));
            root.unwrap()
        };

        let state = TopLevelState::from_existing(db, root).unwrap();
        assert_eq!(Ok(100), state.balance(&a));
        assert_eq!(Ok(1), state.seq(&a));
    }

    #[test]
    fn get_from_cache() {
        let memory_db = get_memory_db();
        let jorunal = new_journaldb(Arc::clone(&memory_db), Algorithm::Archive, Some(0));
        let mut db = StateDB::new(jorunal.boxed_clone());
        let a = Address::default();
        let root = {
            let mut state = empty_top_state_with_metadata(db.clone(&H256::zero()), CommonParams::default_for_test());
            assert_eq!(Ok(()), state.inc_seq(&a));
            assert_eq!(Ok(()), state.add_balance(&a, 69));
            assert_eq!(Ok(69), state.balance(&a));
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(69), state.balance(&a));

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(4, records.unwrap());
            memory_db.write_buffered(transaction);

            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(69), state.balance(&a));

            db.override_state(&state);
            root.unwrap()
        };

        let state = TopLevelState::from_existing(db, root).unwrap();
        assert_eq!(Ok(69), state.balance(&a));
        assert_eq!(Ok(1), state.seq(&a));
    }

    #[test]
    fn remove() {
        let a = Address::default();
        let mut state = get_temp_state();
        assert_eq!(Ok(false), state.account_exists(&a));
        assert_eq!(Ok(false), state.account_exists_and_not_null(&a));
        assert_eq!(Ok(()), state.inc_seq(&a));
        assert_eq!(Ok(true), state.account_exists(&a));
        assert_eq!(Ok(true), state.account_exists_and_not_null(&a));
        assert_eq!(Ok(1), state.seq(&a));
        state.kill_account(&a);
        assert_eq!(Ok(false), state.account_exists(&a));
        assert_eq!(Ok(false), state.account_exists_and_not_null(&a));
        assert_eq!(Ok(0), state.seq(&a));
    }

    #[test]
    fn empty_account_is_not_created() {
        let a = Address::default();
        let mut db = get_temp_state_db();
        let root = {
            let mut state = empty_top_state(db.clone(&H256::zero()));
            assert_eq!(Ok(()), state.add_balance(&a, 0)); // create an empty account
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);

            assert_eq!(Ok(false), state.account_exists(&a));
            assert_eq!(Ok(false), state.account_exists_and_not_null(&a));

            db.override_state(&state);

            root.unwrap()
        };
        let state = TopLevelState::from_existing(db, root).unwrap();
        assert_eq!(Ok(false), state.account_exists(&a));
        assert_eq!(Ok(false), state.account_exists_and_not_null(&a));
    }

    #[test]
    fn remove_from_database() {
        let a = Address::default();
        let memory_db = get_memory_db();
        let jorunal = new_journaldb(Arc::clone(&memory_db), Algorithm::Archive, Some(0));
        let mut db = StateDB::new(jorunal.boxed_clone());
        let root = {
            let mut state = empty_top_state_with_metadata(db.clone(&H256::zero()), CommonParams::default_for_test());
            assert_eq!(Ok(()), state.inc_seq(&a));
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(true), state.account_exists(&a));
            assert_eq!(Ok(1), state.seq(&a));

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(4, records.unwrap());
            memory_db.write_buffered(transaction);

            assert_eq!(Ok(true), state.account_exists(&a));
            assert_eq!(Ok(1), state.seq(&a));

            db.override_state(&state);

            root.unwrap()
        };

        let root = {
            let mut state = TopLevelState::from_existing(db.clone(&root), root).unwrap();
            assert_eq!(Ok(true), state.account_exists(&a));
            assert_eq!(Ok(1), state.seq(&a));
            state.kill_account(&a);
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(false), state.account_exists(&a));
            assert_eq!(Ok(0), state.seq(&a));

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(1, records.unwrap());
            memory_db.write_buffered(transaction);

            assert_eq!(Ok(false), state.account_exists(&a));
            assert_eq!(Ok(0), state.seq(&a));

            db.override_state(&state);

            root.unwrap()
        };

        let state = TopLevelState::from_existing(db, root).unwrap();
        assert_eq!(Ok(false), state.account_exists(&a));
        assert_eq!(Ok(0), state.seq(&a));
    }

    #[test]
    fn alter_balance() {
        let mut state = get_temp_state();
        let a = Address::default();
        let b = 1u64.into();
        assert_eq!(Ok(()), state.add_balance(&a, 100));
        assert_eq!(Ok(100), state.balance(&a));
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        assert_eq!(Ok(100), state.balance(&a));
        assert_eq!(Ok(()), state.sub_balance(&a, 42));
        assert_eq!(Ok(100 - 42), state.balance(&a));
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        assert_eq!(Ok(100 - 42), state.balance(&a));
        assert_eq!(Ok(()), state.transfer_balance(&a, &b, 18));
        assert_eq!(Ok(100 - 42 - 18), state.balance(&a));
        assert_eq!(Ok(18), state.balance(&b));
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        assert_eq!(Ok(100 - 42 - 18), state.balance(&a));
        assert_eq!(Ok(18), state.balance(&b));
    }

    #[test]
    fn alter_seq() {
        let mut state = get_temp_state();
        let a = Address::default();
        assert_eq!(Ok(()), state.inc_seq(&a));
        assert_eq!(Ok(1), state.seq(&a));
        assert_eq!(Ok(()), state.inc_seq(&a));
        assert_eq!(Ok(2), state.seq(&a));
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        assert_eq!(Ok(2), state.seq(&a));
        assert_eq!(Ok(()), state.inc_seq(&a));
        assert_eq!(Ok(3), state.seq(&a));
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        assert_eq!(Ok(3), state.seq(&a));
    }

    #[test]
    fn balance_seq() {
        let mut state = get_temp_state();
        let a = Address::default();
        assert_eq!(Ok(0), state.balance(&a));
        assert_eq!(Ok(0), state.seq(&a));
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        assert_eq!(Ok(0), state.balance(&a));
        assert_eq!(Ok(0), state.seq(&a));
    }

    #[test]
    fn ensure_cached() {
        let mut state = get_temp_state();
        let a = Address::default();
        state.get_account_mut(&a).unwrap();
        assert_eq!(Ok(H256::from("2acc986aa24f6672fe212d16a7f02b559593d148c8b3e073fa18bf47f5abbc72")), state.commit());
    }

    #[test]
    fn checkpoint_basic() {
        let mut state = get_temp_state();
        let a = Address::default();
        StateWithCheckpoint::create_checkpoint(&mut state, 0);
        assert_eq!(Ok(()), state.add_balance(&a, 100));
        assert_eq!(Ok(100), state.balance(&a));
        StateWithCheckpoint::discard_checkpoint(&mut state, 0);
        assert_eq!(Ok(100), state.balance(&a));
        StateWithCheckpoint::create_checkpoint(&mut state, 1);
        assert_eq!(Ok(()), state.add_balance(&a, 1));
        assert_eq!(Ok(100 + 1), state.balance(&a));
        StateWithCheckpoint::revert_to_checkpoint(&mut state, 1);
        assert_eq!(Ok(100), state.balance(&a));
    }

    #[test]
    fn checkpoint_nested() {
        let mut state = get_temp_state();
        let a = Address::default();
        StateWithCheckpoint::create_checkpoint(&mut state, 0);
        assert_eq!(Ok(()), state.add_balance(&a, 100));
        StateWithCheckpoint::create_checkpoint(&mut state, 1);
        assert_eq!(Ok(()), state.add_balance(&a, 120));
        assert_eq!(Ok(100 + 120), state.balance(&a));
        StateWithCheckpoint::revert_to_checkpoint(&mut state, 1);
        assert_eq!(Ok(100), state.balance(&a));
        StateWithCheckpoint::revert_to_checkpoint(&mut state, 0);
        assert_eq!(Ok(0), state.balance(&a));
    }

    #[test]
    fn checkpoint_discard() {
        let mut state = get_temp_state();
        let a = Address::default();
        StateWithCheckpoint::create_checkpoint(&mut state, 0);
        assert_eq!(Ok(()), state.add_balance(&a, 100));
        StateWithCheckpoint::create_checkpoint(&mut state, 1);
        assert_eq!(Ok(()), state.add_balance(&a, 123));
        assert_eq!(Ok(()), state.inc_seq(&a));
        assert_eq!(Ok(100 + 123), state.balance(&a));
        assert_eq!(Ok(1), state.seq(&a));
        StateWithCheckpoint::discard_checkpoint(&mut state, 1);
        assert_eq!(Ok(100 + 123), state.balance(&a));
        assert_eq!(Ok(1), state.seq(&a));
        StateWithCheckpoint::revert_to_checkpoint(&mut state, 0);
        assert_eq!(Ok(0), state.balance(&a));
        assert_eq!(Ok(0), state.seq(&a));
    }
}

#[cfg(test)]
mod tests_tx {
    use ckey::{Ed25519KeyPair as KeyPair, Generator, KeyPairTrait, Random};
    use ctypes::errors::RuntimeError;

    use super::*;
    use crate::tests::helpers::{get_temp_state, get_test_client};
    use crate::StateError;

    fn address() -> (Address, Public) {
        let keypair: KeyPair = Random.generate().unwrap();
        (keypair.address(), *keypair.public())
    }

    #[test]
    fn apply_error_for_invalid_seq() {
        let mut state = get_temp_state();

        let (sender, sender_public) = address();
        set_top_level_state!(state, [(account: sender => balance: 20)]);

        let tx = transaction!(seq: 2, fee: 5, pay!(address().0, 10));
        assert_eq!(
            Err(StateError::Runtime(RuntimeError::InvalidSeq(Mismatch {
                expected: 0,
                found: 2
            }))),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 20))
        ]);
    }

    #[test]
    fn apply_error_for_not_enough_cash() {
        let mut state = get_temp_state();

        let (sender, sender_public) = address();
        set_top_level_state!(state, [(account: sender => balance: 4)]);

        let tx = transaction!(fee: 5, pay!(address().0, 10));
        assert_eq!(
            Err(RuntimeError::InsufficientBalance {
                address: sender,
                balance: 4,
                cost: 5,
            }
            .into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 4))
        ]);
    }

    #[test]
    fn apply_pay() {
        let mut state = get_temp_state();

        let (sender, sender_public) = address();
        set_top_level_state!(state, [(account: sender => balance: 20)]);

        let receiver = 1u64.into();
        let tx = transaction!(fee: 5, pay!(receiver, 10));
        assert_eq!(Ok(()), state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0));

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 5)),
            (account: receiver => (seq: 0, balance: 10))
        ]);
    }

    #[test]
    fn apply_error_for_action_failure() {
        let mut state = get_temp_state();
        let (sender, sender_public) = address();
        set_top_level_state!(state, [
            (account: sender => balance: 20)
        ]);

        let receiver = 1u64.into();
        let tx = transaction!(fee: 5, pay!(receiver, 30));

        assert_eq!(
            Err(RuntimeError::InsufficientBalance {
                address: sender,
                balance: 15,
                cost: 30,
            }
            .into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 20)),
            (account: receiver => (seq: 0, balance: 0))
        ]);
    }
}
