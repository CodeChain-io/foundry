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

use crate::cache::{ModuleCache, ShardCache, TopCache};
use crate::checkpoint::{CheckpointId, StateWithCheckpoint};
use crate::stake::{
    change_params, close_term, delegate_ccs, jail, redelegate, release_jailed_prisoners, revoke, self_nominate,
    transfer_ccs,
};
use crate::traits::{ModuleStateView, ShardState, ShardStateView, StateWithCache, TopState, TopStateView};
use crate::{
    Account, ActionData, CurrentValidators, FindDoubleVoteHandler, Metadata, MetadataAddress, Module, ModuleAddress,
    ModuleLevelState, NextValidators, Shard, ShardAddress, ShardLevelState, StateDB, StateResult,
};
use cdb::{AsHashDB, DatabaseError};
use ckey::{Ed25519Public as Public, NetworkId};
use ctypes::errors::RuntimeError;
use ctypes::transaction::{Action, ShardTransaction, Transaction};
use ctypes::util::unexpected::Mismatch;
use ctypes::{BlockNumber, CommonParams, ShardId, StorageId, TxHash};
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
    shard_caches: HashMap<ShardId, ShardCache>,
    module_caches: HashMap<StorageId, ModuleCache>,
    id_of_checkpoints: Vec<CheckpointId>,
}

impl TopStateView for TopLevelState {
    /// Check caches for required data
    /// First searches for account in the local, then the shared cache.
    /// Populates local cache if nothing found.
    fn account(&self, a: &Public) -> TrieResult<Option<Account>> {
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

    fn shard(&self, shard_id: ShardId) -> TrieResult<Option<Shard>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let shard_address = ShardAddress::new(shard_id);
        self.top_cache.shard(&shard_address, &trie)
    }

    fn module(&self, storage_id: StorageId) -> TrieResult<Option<Module>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let module_address = ModuleAddress::new(storage_id);
        self.top_cache.module(&module_address, &trie)
    }

    fn shard_state<'db>(&'db self, shard_id: ShardId) -> TrieResult<Option<Box<dyn ShardStateView + 'db>>> {
        match self.shard_root(shard_id)? {
            // FIXME: Find a way to use stored cache.
            Some(shard_root) => {
                let shard_cache = self.shard_caches.get(&shard_id).cloned().unwrap_or_default();
                Ok(Some(Box::new(ShardLevelState::read_only(shard_id, &self.db, shard_root, shard_cache)?)))
            }
            None => Ok(None),
        }
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

impl StateWithCache for TopLevelState {
    fn commit(&mut self) -> StateResult<H256> {
        let shard_ids: Vec<_> = self.shard_caches.iter().map(|(shard_id, _)| *shard_id).collect();
        let shard_changes = shard_ids
            .into_iter()
            .map(|shard_id| {
                if let Some(shard_root) = self.shard_root(shard_id)? {
                    Ok(Some((shard_id, shard_root)))
                } else {
                    Ok(None)
                }
            })
            .collect::<StateResult<Vec<_>>>()?;
        for (shard_id, mut shard_root) in shard_changes.into_iter().filter_map(|result| result) {
            {
                let mut db = self.db.borrow_mut();
                let mut trie = TrieFactory::from_existing(db.as_hashdb_mut(), &mut shard_root)?;

                let shard_cache = self.shard_caches.get_mut(&shard_id).expect("Shard must exist");

                shard_cache.commit(&mut trie)?;
            }
            self.set_shard_root(shard_id, shard_root)?;
        }
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

const FEE_CHECKPOINT: CheckpointId = 123;
const ACTION_CHECKPOINT: CheckpointId = 130;

impl StateWithCheckpoint for TopLevelState {
    fn create_checkpoint(&mut self, id: CheckpointId) {
        ctrace!(STATE, "Checkpoint({}) for top level is created", id);
        self.id_of_checkpoints.push(id);
        self.top_cache.checkpoint();

        for (_, cache) in self.shard_caches.iter_mut() {
            cache.checkpoint()
        }
        self.module_caches.iter_mut().for_each(|(_, cache)| cache.checkpoint())
    }

    fn discard_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is discarded", id);
        self.top_cache.discard_checkpoint();

        for (_, cache) in self.shard_caches.iter_mut() {
            cache.discard_checkpoint();
        }
        self.module_caches.iter_mut().for_each(|(_, cache)| cache.discard_checkpoint())
    }

    fn revert_to_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is reverted", id);
        self.top_cache.revert_to_checkpoint();

        for (_, cache) in self.shard_caches.iter_mut() {
            cache.revert_to_checkpoint();
        }
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
        let shard_caches = db.shard_caches();
        let module_caches = db.module_caches();

        let state = TopLevelState {
            db: RefCell::new(db),
            root,
            top_cache,
            shard_caches,
            module_caches,
            id_of_checkpoints: Default::default(),
        };

        Ok(state)
    }

    /// Execute a given tranasction, charging tranasction fee.
    /// This will change the state accordingly.
    pub fn apply<C: FindDoubleVoteHandler>(
        &mut self,
        tx: &Transaction,
        sender_public: &Public,
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
        current_block_timestamp: u64,
    ) -> StateResult<()> {
        self.create_checkpoint(FEE_CHECKPOINT);
        let result = self.apply_internal(
            tx,
            sender_public,
            client,
            parent_block_number,
            parent_block_timestamp,
            current_block_timestamp,
        );
        match result {
            Ok(()) => {
                self.discard_checkpoint(FEE_CHECKPOINT);
            }
            Err(_) => {
                self.revert_to_checkpoint(FEE_CHECKPOINT);
            }
        }
        result
    }

    fn apply_internal<C: FindDoubleVoteHandler>(
        &mut self,
        tx: &Transaction,
        sender: &Public,
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
        current_block_timestamp: u64,
    ) -> StateResult<()> {
        let seq = self.seq(sender)?;

        if tx.seq != seq {
            return Err(RuntimeError::InvalidSeq(Mismatch {
                expected: seq,
                found: tx.seq,
            })
            .into())
        }

        let fee = tx.fee;

        self.inc_seq(sender)?;
        self.sub_balance(sender, fee)?;

        // The failed transaction also must pay the fee and increase seq.
        self.create_checkpoint(ACTION_CHECKPOINT);
        let result = self.apply_action(
            &tx.action,
            tx.network_id,
            tx.hash(),
            sender,
            client,
            parent_block_number,
            parent_block_timestamp,
            current_block_timestamp,
        );
        match &result {
            Ok(()) => {
                self.discard_checkpoint(ACTION_CHECKPOINT);
            }
            Err(_) => {
                self.revert_to_checkpoint(ACTION_CHECKPOINT);
            }
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_action<C: FindDoubleVoteHandler>(
        &mut self,
        action: &Action,
        network_id: NetworkId,
        tx_hash: TxHash,
        sender: &Public,
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
        _current_block_timestamp: u64,
    ) -> StateResult<()> {
        let (transaction, approvers) = match action {
            Action::ShardStore {
                ..
            } => {
                let transaction = Option::<ShardTransaction>::from(action.clone()).expect("It's a shard transaction");
                debug_assert_eq!(network_id, transaction.network_id());

                let approvers = vec![]; // WrapCCC doesn't have approvers
                (transaction, approvers)
            }
            Action::Pay {
                receiver,
                quantity,
            } => {
                self.transfer_balance(sender, receiver, *quantity)?;
                return Ok(())
            }
            Action::TransferCCS {
                address,
                quantity,
            } => return transfer_ccs(self, sender, &address, *quantity),
            Action::DelegateCCS {
                address,
                quantity,
            } => return delegate_ccs(self, sender, &address, *quantity),
            Action::Revoke {
                address,
                quantity,
            } => return revoke(self, sender, address, *quantity),
            Action::Redelegate {
                prev_delegatee,
                next_delegatee,
                quantity,
            } => return redelegate(self, sender, prev_delegatee, next_delegatee, *quantity),
            Action::SelfNominate {
                deposit,
                metadata,
            } => {
                let (current_term, nomination_ends_at) = {
                    let metadata = self.metadata()?.expect("Metadata must exist");
                    let current_term = metadata.current_term_id();
                    let expiration = metadata.params().nomination_expiration();
                    let nomination_ends_at = current_term + expiration;
                    (current_term, nomination_ends_at)
                };
                return self_nominate(self, sender, *deposit, current_term, nomination_ends_at, metadata.clone())
            }
            Action::ChangeParams {
                metadata_seq,
                params,
                approvals,
            } => return change_params(self, *metadata_seq, **params, &approvals),
            Action::ReportDoubleVote {
                message1,
                ..
            } => {
                let handler = client.double_vote_handler().expect("Unknown custom transaction applied!");
                handler.execute(message1, self, sender)?;
                return Ok(())
            }
            Action::UpdateValidators {
                validators,
            } => {
                let next_validators_in_state = NextValidators::load_from_state(self)?;
                if validators != &Vec::from(next_validators_in_state) {
                    return Err(RuntimeError::InvalidValidators.into())
                }
                let mut current_validators = CurrentValidators::load_from_state(self)?;
                current_validators.update(validators.clone());
                current_validators.save_to_state(self)?;
                return Ok(())
            }
            Action::CloseTerm {
                inactive_validators,
                next_validators,
                released_addresses,
                custody_until,
                kick_at,
            } => {
                close_term(self, next_validators, inactive_validators)?;
                release_jailed_prisoners(self, released_addresses)?;
                jail(self, inactive_validators, *custody_until, *kick_at)?;
                return self.increase_term_id(parent_block_number + 1)
            }
            Action::ChangeNextValidators {
                validators,
            } => return NextValidators::from(validators.clone()).save_to_state(self),
            Action::Elect => {
                NextValidators::elect(self)?.save_to_state(self)?;
                return self.update_term_params()
            }
        };
        self.apply_shard_transaction(
            tx_hash,
            &transaction,
            sender,
            &approvers,
            parent_block_number,
            parent_block_timestamp,
        )
    }

    fn apply_shard_transaction(
        &mut self,
        tx_hash: TxHash,
        transaction: &ShardTransaction,
        sender: &Public,
        approvers: &[Public],
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
    ) -> StateResult<()> {
        for shard_id in transaction.related_shards() {
            self.apply_shard_transaction_to_shard(
                tx_hash,
                transaction,
                shard_id,
                sender,
                approvers,
                parent_block_number,
                parent_block_timestamp,
            )?;
        }
        Ok(())
    }

    fn apply_shard_transaction_to_shard(
        &mut self,
        tx_hash: TxHash,
        transaction: &ShardTransaction,
        shard_id: ShardId,
        sender: &Public,
        approvers: &[Public],
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
    ) -> StateResult<()> {
        let shard_root = self.shard_root(shard_id)?.ok_or_else(|| RuntimeError::InvalidShardId(shard_id))?;

        let shard_cache = self.shard_caches.entry(shard_id).or_default();
        let mut shard_level_state = ShardLevelState::from_existing(shard_id, &mut self.db, shard_root, shard_cache)?;
        shard_level_state.apply(tx_hash, &transaction, sender, approvers, parent_block_number, parent_block_timestamp)
    }

    fn create_module_level_state(&mut self, storage_id: StorageId) -> StateResult<()> {
        const DEFAULT_MODULE_ROOT: H256 = ccrypto::BLAKE_NULL_RLP;
        {
            let module_cache = self.module_caches.entry(storage_id).or_default();
            ModuleLevelState::from_existing(storage_id, &mut self.db, DEFAULT_MODULE_ROOT, module_cache)?;
        }
        ctrace!(STATE, "module storage({}) created", storage_id);
        self.set_module_root(storage_id, DEFAULT_MODULE_ROOT)?;
        Ok(())
    }

    #[allow(dead_code)]
    fn module_state_mut(&mut self, storage_id: StorageId) -> StateResult<ModuleLevelState> {
        let module_root = self.module_root(storage_id)?.ok_or_else(|| RuntimeError::InvalidStorageId(storage_id))?;
        let module_cache = self.module_caches.entry(storage_id).or_default();
        Ok(ModuleLevelState::from_existing(storage_id, &mut self.db, module_root, module_cache)?)
    }

    fn get_account_mut(&self, a: &Public) -> TrieResult<RefMut<'_, Account>> {
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

    fn get_shard_mut(&self, shard_id: ShardId) -> TrieResult<RefMut<'_, Shard>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let shard_address = ShardAddress::new(shard_id);
        self.top_cache.shard_mut(&shard_address, &trie)
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
    pub fn shard_caches(&self) -> &HashMap<ShardId, ShardCache> {
        &self.shard_caches
    }

    pub fn module_caches(&self) -> &HashMap<StorageId, ModuleCache> {
        &self.module_caches
    }

    pub fn root(&self) -> H256 {
        self.root
    }

    #[cfg(test)]
    fn set_balance(&mut self, a: &Public, balance: u64) -> TrieResult<()> {
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
            shard_caches: self.shard_caches.clone(),
            module_caches: self.module_caches.clone(),
        }
    }
}

impl TopState for TopLevelState {
    fn kill_account(&mut self, account: &Public) {
        self.top_cache.remove_account(account);
    }

    fn add_balance(&mut self, a: &Public, incr: u64) -> TrieResult<()> {
        ctrace!(STATE, "add_balance({:?}, {}): {}", a, incr, self.balance(a)?);
        if incr != 0 {
            self.get_account_mut(a)?.add_balance(incr);
        }
        Ok(())
    }

    fn sub_balance(&mut self, a: &Public, decr: u64) -> StateResult<()> {
        ctrace!(STATE, "sub_balance({:?}, {}): {}", a, decr, self.balance(a)?);
        if decr == 0 {
            return Ok(())
        }
        let balance = self.balance(a)?;
        if balance < decr {
            return Err(RuntimeError::InsufficientBalance {
                pubkey: *a,
                cost: decr,
                balance,
            }
            .into())
        }
        self.get_account_mut(a)?.sub_balance(decr);
        Ok(())
    }

    fn transfer_balance(&mut self, from: &Public, to: &Public, by: u64) -> StateResult<()> {
        self.sub_balance(from, by)?;
        self.add_balance(to, by)?;
        Ok(())
    }

    fn inc_seq(&mut self, a: &Public) -> TrieResult<()> {
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

    fn set_shard_root(&mut self, shard_id: ShardId, new_root: H256) -> StateResult<()> {
        let mut shard = self.get_shard_mut(shard_id)?;
        shard.set_root(new_root);
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
        let a = Public::default();

        let mut state = {
            let mut state = get_temp_state();
            top_level!(state, {
                check: [(account: a => None)],
                set: [(account: a => seq++)],
                check: [(account: a => seq: 1)],
            });
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            state
        };
        top_level!(state, {
            check: [(account: a => seq: 1)],
            set: [(account: a => seq++)],
            check: [(account: a => seq: 2)],
        });
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        top_level!(state, {
            check: [(account: a => seq: 2)]
        })
    }

    #[test]
    fn work_when_cloned_even_not_committed() {
        let a = Public::default();

        let mut state = {
            let mut state = get_temp_state();
            top_level!(state, {
                check: [(account: a => None)],
                set: [(account: a => seq++)],
                check: [(account: a => seq: 1)],
            });
            state
        };
        top_level!(state, {
            check: [(account: a => seq: 1)],
            set: [(account: a => seq++)],
            check: [(account: a => seq: 2)],
        });
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        top_level!(state, {
            check: [(account: a => seq: 2)]
        })
    }

    #[test]
    fn state_is_not_synchronized_when_cloned() {
        let a = Public::random();

        let original_state = get_temp_state();
        top_level!(original_state, {
            check: [(account: a => None)]
        });
        let mut cloned_state = original_state.clone();
        top_level!(cloned_state, {
            set: [(account: a => seq++)]
        });
        let root = cloned_state.commit();
        assert!(root.is_ok(), "{:?}", root);

        assert_ne!(original_state.seq(&a), cloned_state.seq(&a));
    }

    #[test]
    fn get_from_database() {
        let memory_db = get_memory_db();
        let jorunal = new_journaldb(Arc::clone(&memory_db), Algorithm::Archive, Some(0));
        let db = StateDB::new(jorunal.boxed_clone());
        let a = Public::default();
        let root = {
            let mut state = empty_top_state_with_metadata(db.clone(&H256::zero()), CommonParams::default_for_test());
            top_level!(state, {
                set: [
                    (account: a => seq++),
                    (account: a => balance: add 100)
                ],
                check: [(account: a => balance: 100)]
            });
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            top_level!(state, {
                check: [(account: a => balance: 100)]
            });

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(4, records.unwrap());
            memory_db.write_buffered(transaction);

            assert!(root.is_ok(), "{:?}", root);
            top_level!(state, {
                check: [(account: a => balance: 100)]
            });
            root.unwrap()
        };

        let state = TopLevelState::from_existing(db, root).unwrap();
        top_level!(state, {
            check: [(account: a => balance: 100, seq: 1)]
        });
    }

    #[test]
    fn get_from_cache() {
        let memory_db = get_memory_db();
        let jorunal = new_journaldb(Arc::clone(&memory_db), Algorithm::Archive, Some(0));
        let mut db = StateDB::new(jorunal.boxed_clone());
        let a = Public::default();
        let root = {
            let mut state = empty_top_state_with_metadata(db.clone(&H256::zero()), CommonParams::default_for_test());
            top_level!(state, {
                set: [
                    (account: a => seq++),
                    (account: a => balance: add 69)
                ],
                check: [
                    (account: a => balance: 69)
                ]
            });
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            top_level!(state, {
                check: [(account: a => balance: 69)]
            });

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(4, records.unwrap());
            memory_db.write_buffered(transaction);

            assert!(root.is_ok(), "{:?}", root);
            top_level!(state, {
                check: [(account: a => balance: 69)]
            });

            db.override_state(&state);
            root.unwrap()
        };

        let state = TopLevelState::from_existing(db, root).unwrap();
        top_level!(state, {
            check: [(account: a => balance: 69, seq: 1)]
        });
    }

    #[test]
    fn remove() {
        let a = Public::default();
        let mut state = get_temp_state();
        top_level!(state, {
            check: [(account: a => None)],
            set: [(account: a => seq++)],
            check: [(account: a => Some), (account: a => seq: 1)],
            set: [(account: a => Kill)],
            check: [(account: a => None), (account: a => seq: 0)],
        });
    }

    #[test]
    fn empty_account_is_not_created() {
        let a = Public::default();
        let mut db = get_temp_state_db();
        let root = {
            let mut state = empty_top_state(db.clone(&H256::zero()));
            top_level!(state, {
                set: [(account: a => balance: add 0)] // create an empty account
            });
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);

            top_level!(state, {
                check: [(account: a => None)]
            });

            db.override_state(&state);

            root.unwrap()
        };
        let state = TopLevelState::from_existing(db, root).unwrap();
        top_level!(state, {
            check: [(account: a => None)]
        });
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn remove_from_database() {
        let a = Public::default();
        let memory_db = get_memory_db();
        let jorunal = new_journaldb(Arc::clone(&memory_db), Algorithm::Archive, Some(0));
        let mut db = StateDB::new(jorunal.boxed_clone());
        let root = {
            let mut state = empty_top_state_with_metadata(db.clone(&H256::zero()), CommonParams::default_for_test());
            top_level!(state, {
                set: [(account: a => seq++)]
            });
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            top_level!(state, {
                check: [(account: a => Some), (account: a => seq: 1)]
            });

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(4, records.unwrap());
            memory_db.write_buffered(transaction);

            top_level!(state, {
                check: [(account: a => Some), (account: a => seq: 1)]
            });

            db.override_state(&state);

            root.unwrap()
        };

        let root = {
            let mut state = TopLevelState::from_existing(db.clone(&root), root).unwrap();
            top_level!(state, {
                check: [(account: a => Some), (account: a => seq: 1)],
                set: [(account: a => Kill)],
            });
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            top_level!(state, {
                check: [(account: a => None), (account: a => seq: 0)]
            });

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(1, records.unwrap());
            memory_db.write_buffered(transaction);

            top_level!(state, {
                check: [(account: a => None), (account: a => seq: 0)]
            });

            db.override_state(&state);

            root.unwrap()
        };

        let state = TopLevelState::from_existing(db, root).unwrap();
        top_level!(state, {
            check: [(account: a => None), (account: a => seq: 0)]
        });
    }

    #[test]
    fn alter_balance() {
        let mut state = get_temp_state();
        let a = Public::default();
        let b = 1u64.into();
        top_level!(state, {
            set: [(account: a => balance: add 100)],
            check: [(account: a => balance: 100)],
        });
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        top_level!(state, {
            check: [(account: a => balance: 100)],
            set: [(account: a => balance: sub 42)],
            check: [(account: a => balance: 100 - 42)],
        });
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        top_level!(state, {
            check: [(account: a => balance: 100 - 42)],
            set: [(account: a => transfer: b, 18)],
            check: [
                (account: a => balance: 100 - 42 - 18),
                (account: b => balance: 18),
            ],
        });
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        top_level!(state, {
            check: [
                (account: a => balance: 100 - 42 - 18),
                (account: b => balance: 18),
            ]
        });
    }

    #[test]
    fn alter_seq() {
        let mut state = get_temp_state();
        let a = Public::default();
        top_level!(state, {
            set: [(account: a => seq++)],
            check: [(account: a => seq: 1)],
            set: [(account: a => seq++)],
            check: [(account: a => seq: 2)],
        });
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        top_level!(state, {
            check: [(account: a => seq: 2)],
            set: [(account: a => seq++)],
            check: [(account: a => seq: 3)],
        });
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        top_level!(state, {
            check: [(account: a => seq: 3)]
        });
    }

    #[test]
    fn balance_seq() {
        let mut state = get_temp_state();
        let a = Public::default();
        top_level!(state, {
            check: [(account: a => balance: 0, seq: 0)]
        });
        let root = state.commit();
        assert!(root.is_ok(), "{:?}", root);
        top_level!(state, {
            check: [(account: a => balance: 0, seq: 0)]
        });
    }

    #[test]
    fn ensure_cached() {
        let mut state = get_temp_state();
        let a = Public::default();
        state.get_account_mut(&a).unwrap();
        assert_eq!(Ok(H256::from("00f286cd9d2e09f0881b802a33af986e58039506c3e84e0e76d7fa58d6c0adfe")), state.commit());
    }

    #[test]
    fn checkpoint_basic() {
        let mut state = get_temp_state();
        let a = Public::default();
        state.create_checkpoint(0);
        top_level!(state, {
            set: [(account: a => balance: add 100)],
            check: [(account: a => balance: 100)]
        });
        state.discard_checkpoint(0);
        top_level!(state, {
            check: [(account: a => balance: 100)]
        });
        state.create_checkpoint(1);
        top_level!(state, {
            set: [(account: a => balance: add 1)],
            check: [(account: a => balance: 100 + 1)]
        });
        state.revert_to_checkpoint(1);
        top_level!(state, {
            check: [(account: a => balance: 100)]
        });
    }

    #[test]
    fn checkpoint_nested() {
        let mut state = get_temp_state();
        let a = Public::default();
        state.create_checkpoint(0);
        top_level!(state, {
            set: [(account: a => balance: add 100)]
        });
        state.create_checkpoint(1);
        top_level!(state, {
            set: [(account: a => balance: add 120)],
            check: [(account: a => balance: 100 + 120)]
        });
        state.revert_to_checkpoint(1);
        top_level!(state, {
            check: [(account: a => balance: 100)]
        });
        state.revert_to_checkpoint(0);
        top_level!(state, {
            check: [(account: a => balance: 0)]
        });
    }

    #[test]
    fn checkpoint_discard() {
        let mut state = get_temp_state();
        let a = Public::default();
        state.create_checkpoint(0);
        top_level!(state, {
            set: [(account: a => balance: add 100)]
        });
        state.create_checkpoint(1);
        top_level!(state, {
            set: [(account: a => balance: add 123), (account: a => seq++)],
            check: [(account: a => balance: 100 + 123, seq: 1)]
        });
        state.discard_checkpoint(1);
        top_level!(state, {
            check: [(account: a => balance: 100 + 123, seq: 1)]
        });
        state.revert_to_checkpoint(0);
        top_level!(state, {
            check: [(account: a => balance: 0, seq: 0)]
        });
    }
}

#[cfg(test)]
mod tests_tx {
    use ckey::{Ed25519KeyPair as KeyPair, Generator, KeyPairTrait, Random};
    use ctypes::errors::RuntimeError;

    use super::*;
    use crate::tests::helpers::{get_temp_state, get_test_client};
    use crate::StateError;

    fn random_pubkey() -> Public {
        let keypair: KeyPair = Random.generate().unwrap();
        *keypair.public()
    }

    #[test]
    fn apply_error_for_invalid_seq() {
        let mut state = get_temp_state();

        let sender = random_pubkey();
        set_top_level_state!(state, [(account: sender => balance: 20)]);

        let tx = transaction!(seq: 2, fee: 5, pay!(random_pubkey(), 10));
        assert_eq!(
            Err(StateError::Runtime(RuntimeError::InvalidSeq(Mismatch {
                expected: 0,
                found: 2
            }))),
            state.apply(&tx, &sender, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => seq: 0, balance: 20)
        ]);
    }

    #[test]
    fn apply_error_for_not_enough_cash() {
        let mut state = get_temp_state();

        let sender = random_pubkey();
        set_top_level_state!(state, [(account: sender => balance: 4)]);

        let tx = transaction!(fee: 5, pay!(random_pubkey(), 10));
        assert_eq!(
            Err(RuntimeError::InsufficientBalance {
                pubkey: sender,
                balance: 4,
                cost: 5,
            }
            .into()),
            state.apply(&tx, &sender, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => seq: 0, balance: 4)
        ]);
    }

    #[test]
    fn apply_pay() {
        let mut state = get_temp_state();

        let sender = random_pubkey();
        set_top_level_state!(state, [(account: sender => balance: 20)]);

        let receiver = 1u64.into();
        let tx = transaction!(fee: 5, pay!(receiver, 10));
        assert_eq!(Ok(()), state.apply(&tx, &sender, &get_test_client(), 0, 0, 0));

        check_top_level_state!(state, [
            (account: sender => seq: 1, balance: 5),
            (account: receiver => seq: 0, balance: 10)
        ]);
    }

    #[test]
    fn apply_error_for_action_failure() {
        let mut state = get_temp_state();
        let sender = random_pubkey();
        set_top_level_state!(state, [
            (account: sender => balance: 20)
        ]);

        let receiver = 1u64.into();
        let tx = transaction!(fee: 5, pay!(receiver, 30));

        assert_eq!(
            Err(RuntimeError::InsufficientBalance {
                pubkey: sender,
                balance: 15,
                cost: 30,
            }
            .into()),
            state.apply(&tx, &sender, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => seq: 0, balance: 20),
            (account: receiver => seq: 0, balance: 0)
        ]);
    }

    #[test]
    fn get_invalid_shard_root() {
        let state = get_temp_state();

        let shard_id = 3;
        check_top_level_state!(state, [(shard: shard_id)]);
    }

    #[test]
    fn get_shard_text_in_invalid_shard() {
        let state = get_temp_state();

        let shard_id = 3;
        check_top_level_state!(state, [(shard_text: (shard_id, TxHash::from(H256::random())))]);
    }
}
