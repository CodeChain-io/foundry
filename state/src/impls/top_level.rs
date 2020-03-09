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

use crate::cache::{ShardCache, TopCache};
use crate::checkpoint::{CheckpointId, StateWithCheckpoint};
use crate::traits::{ShardState, ShardStateView, StateWithCache, TopState, TopStateView};
use crate::{
    Account, ActionData, FindActionHandler, Metadata, MetadataAddress, Shard, ShardAddress, ShardLevelState, StateDB,
    StateResult,
};
use ccrypto::BLAKE_NULL_RLP;
use cdb::{AsHashDB, DatabaseError};
use ckey::{public_to_address, Address, Ed25519Public as Public, NetworkId};
use ctypes::errors::RuntimeError;
use ctypes::transaction::{Action, ShardTransaction, Transaction};
use ctypes::util::unexpected::Mismatch;
#[cfg(test)]
use ctypes::Tracker;
use ctypes::{BlockNumber, CommonParams, ShardId, TxHash};
use cvm::ChainTimeInfo;
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

    fn shard(&self, shard_id: ShardId) -> TrieResult<Option<Shard>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let shard_address = ShardAddress::new(shard_id);
        self.top_cache.shard(&shard_address, &trie)
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
    }

    fn discard_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is discarded", id);
        self.top_cache.discard_checkpoint();

        for (_, cache) in self.shard_caches.iter_mut() {
            cache.discard_checkpoint();
        }
    }

    fn revert_to_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is reverted", id);
        self.top_cache.revert_to_checkpoint();

        for (_, cache) in self.shard_caches.iter_mut() {
            cache.revert_to_checkpoint();
        }
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

        let state = TopLevelState {
            db: RefCell::new(db),
            root,
            top_cache,
            shard_caches,
            id_of_checkpoints: Default::default(),
        };

        Ok(state)
    }

    /// Execute a given tranasction, charging tranasction fee.
    /// This will change the state accordingly.
    pub fn apply<C: ChainTimeInfo + FindActionHandler>(
        &mut self,
        tx: &Transaction,
        signed_hash: &TxHash,
        sender_public: &Public,
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
        current_block_timestamp: u64,
    ) -> StateResult<()> {
        self.create_checkpoint(FEE_CHECKPOINT);
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
                self.discard_checkpoint(FEE_CHECKPOINT);
            }
            Err(_) => {
                self.revert_to_checkpoint(FEE_CHECKPOINT);
            }
        }
        result
    }

    fn apply_internal<C: ChainTimeInfo + FindActionHandler>(
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
        self.create_checkpoint(ACTION_CHECKPOINT);
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
                self.discard_checkpoint(ACTION_CHECKPOINT);
            }
            Err(_) => {
                self.revert_to_checkpoint(ACTION_CHECKPOINT);
            }
        }
        result
    }

    #[allow(clippy::too_many_arguments)]
    fn apply_action<C: ChainTimeInfo + FindActionHandler>(
        &mut self,
        action: &Action,
        network_id: NetworkId,
        _tx_hash: TxHash,
        signed_hash: &TxHash,
        sender_address: &Address,
        sender_public: &Public,
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
                self.transfer_balance(sender_address, receiver, *quantity)?;
                return Ok(())
            }
            Action::CreateShard {
                users,
            } => {
                self.create_shard(sender_address, *signed_hash, users.clone())?;
                return Ok(())
            }
            Action::SetShardOwners {
                shard_id,
                owners,
            } => {
                self.change_shard_owners(*shard_id, owners, sender_address)?;
                return Ok(())
            }
            Action::SetShardUsers {
                shard_id,
                users,
            } => {
                self.change_shard_users(*shard_id, users, sender_address)?;
                return Ok(())
            }
            Action::Custom {
                handler_id,
                bytes,
            } => {
                let handler = client.find_action_handler_for(*handler_id).expect("Unknown custom parsel applied!");
                handler.execute(bytes, self, sender_address, sender_public)?;
                return Ok(())
            }
        };
        self.apply_shard_transaction(
            &transaction,
            sender_address,
            &approvers,
            client,
            parent_block_number,
            parent_block_timestamp,
        )
    }

    pub fn apply_shard_transaction<C: ChainTimeInfo>(
        &mut self,
        transaction: &ShardTransaction,
        sender: &Address,
        approvers: &[Address],
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
    ) -> StateResult<()> {
        for shard_id in transaction.related_shards() {
            self.apply_shard_transaction_to_shard(
                transaction,
                shard_id,
                sender,
                approvers,
                client,
                parent_block_number,
                parent_block_timestamp,
            )?;
        }
        Ok(())
    }

    fn apply_shard_transaction_to_shard<C: ChainTimeInfo>(
        &mut self,
        transaction: &ShardTransaction,
        shard_id: ShardId,
        sender: &Address,
        approvers: &[Address],
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
    ) -> StateResult<()> {
        let shard_root = self.shard_root(shard_id)?.ok_or_else(|| RuntimeError::InvalidShardId(shard_id))?;
        let shard_users = self.shard_users(shard_id)?.expect("Shard must exist");

        let shard_cache = self.shard_caches.entry(shard_id).or_default();
        let mut shard_level_state = ShardLevelState::from_existing(shard_id, &mut self.db, shard_root, shard_cache)?;
        shard_level_state.apply(
            &transaction,
            sender,
            &shard_users,
            approvers,
            client,
            parent_block_number,
            parent_block_timestamp,
        )
    }

    fn create_shard_level_state(
        &mut self,
        shard_id: ShardId,
        owners: Vec<Address>,
        users: Vec<Address>,
    ) -> StateResult<()> {
        const DEFAULT_SHARD_ROOT: H256 = BLAKE_NULL_RLP;
        {
            let shard_cache = self.shard_caches.entry(shard_id).or_default();
            ShardLevelState::from_existing(shard_id, &mut self.db, DEFAULT_SHARD_ROOT, shard_cache)?;
        }

        ctrace!(STATE, "shard({}) created. owners: {:?}, users: {:?}", shard_id, owners, users);

        self.set_shard_root(shard_id, DEFAULT_SHARD_ROOT)?;
        self.set_shard_owners(shard_id, owners)?;
        self.set_shard_users(shard_id, users)?;
        Ok(())
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

    pub fn root(&self) -> H256 {
        self.root
    }

    #[cfg(test)]
    fn set_balance(&mut self, a: &Address, balance: u64) -> TrieResult<()> {
        self.get_account_mut(a)?.set_balance(balance);
        Ok(())
    }
    #[cfg(test)]
    fn set_seq(&mut self, a: &Address, seq: u64) -> TrieResult<()> {
        self.get_account_mut(a)?.set_seq(seq);
        Ok(())
    }

    #[cfg(test)]
    fn set_number_of_shards(&mut self, number_of_shards: ShardId) -> TrieResult<()> {
        self.get_metadata_mut()?.set_number_of_shards(number_of_shards);
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

    fn create_shard(&mut self, fee_payer: &Address, tx_hash: TxHash, users: Vec<Address>) -> StateResult<()> {
        let shard_id = {
            let mut metadata = self.get_metadata_mut()?;
            metadata.add_shard(tx_hash)
        };
        self.create_shard_level_state(shard_id, vec![*fee_payer], users)?;

        Ok(())
    }

    fn change_shard_owners(&mut self, shard_id: ShardId, owners: &[Address], sender: &Address) -> StateResult<()> {
        let old_owners = self.shard_owners(shard_id)?.ok_or_else(|| RuntimeError::InvalidShardId(shard_id))?;
        if !old_owners.contains(sender) {
            return Err(RuntimeError::InsufficientPermission.into())
        }
        if !owners.contains(sender) {
            return Err(RuntimeError::NewOwnersMustContainSender.into())
        }

        self.set_shard_owners(shard_id, owners.to_vec())
    }

    fn change_shard_users(&mut self, shard_id: ShardId, users: &[Address], sender: &Address) -> StateResult<()> {
        let owners = self.shard_owners(shard_id)?.ok_or_else(|| RuntimeError::InvalidShardId(shard_id))?;
        if !owners.contains(sender) {
            return Err(RuntimeError::InsufficientPermission.into())
        }

        self.set_shard_users(shard_id, users.to_vec())
    }

    fn set_shard_root(&mut self, shard_id: ShardId, new_root: H256) -> StateResult<()> {
        let mut shard = self.get_shard_mut(shard_id)?;
        shard.set_root(new_root);
        Ok(())
    }

    fn set_shard_owners(&mut self, shard_id: ShardId, new_owners: Vec<Address>) -> StateResult<()> {
        for owner in &new_owners {
            if !is_active_account(self, owner)? {
                return Err(RuntimeError::NonActiveAccount {
                    name: "shard owner".to_string(),
                    address: *owner,
                }
                .into())
            }
        }
        let mut shard = self.get_shard_mut(shard_id)?;
        shard.set_owners(new_owners);
        Ok(())
    }

    fn set_shard_users(&mut self, shard_id: ShardId, new_users: Vec<Address>) -> StateResult<()> {
        for user in &new_users {
            if !is_active_account(self, user)? {
                return Err(RuntimeError::NonActiveAccount {
                    name: "shard user".to_string(),
                    address: *user,
                }
                .into())
            }
        }
        let mut shard = self.get_shard_mut(shard_id)?;
        shard.set_users(new_users);
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

fn is_active_account(state: &dyn TopStateView, address: &Address) -> TrieResult<bool> {
    match &state.account(address)? {
        Some(account) if account.is_active() => Ok(true),
        _ => Ok(false),
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
        assert_eq!(Ok(H256::from("37db9832f9e2f164789ddf7e399481a0386f61acb49a52d975466058bc1bbbcb")), state.commit());
    }

    #[test]
    fn checkpoint_basic() {
        let mut state = get_temp_state();
        let a = Address::default();
        state.create_checkpoint(0);
        assert_eq!(Ok(()), state.add_balance(&a, 100));
        assert_eq!(Ok(100), state.balance(&a));
        state.discard_checkpoint(0);
        assert_eq!(Ok(100), state.balance(&a));
        state.create_checkpoint(1);
        assert_eq!(Ok(()), state.add_balance(&a, 1));
        assert_eq!(Ok(100 + 1), state.balance(&a));
        state.revert_to_checkpoint(1);
        assert_eq!(Ok(100), state.balance(&a));
    }

    #[test]
    fn checkpoint_nested() {
        let mut state = get_temp_state();
        let a = Address::default();
        state.create_checkpoint(0);
        assert_eq!(Ok(()), state.add_balance(&a, 100));
        state.create_checkpoint(1);
        assert_eq!(Ok(()), state.add_balance(&a, 120));
        assert_eq!(Ok(100 + 120), state.balance(&a));
        state.revert_to_checkpoint(1);
        assert_eq!(Ok(100), state.balance(&a));
        state.revert_to_checkpoint(0);
        assert_eq!(Ok(0), state.balance(&a));
    }

    #[test]
    fn checkpoint_discard() {
        let mut state = get_temp_state();
        let a = Address::default();
        state.create_checkpoint(0);
        assert_eq!(Ok(()), state.add_balance(&a, 100));
        state.create_checkpoint(1);
        assert_eq!(Ok(()), state.add_balance(&a, 123));
        assert_eq!(Ok(()), state.inc_seq(&a));
        assert_eq!(Ok(100 + 123), state.balance(&a));
        assert_eq!(Ok(1), state.seq(&a));
        state.discard_checkpoint(1);
        assert_eq!(Ok(100 + 123), state.balance(&a));
        assert_eq!(Ok(1), state.seq(&a));
        state.revert_to_checkpoint(0);
        assert_eq!(Ok(0), state.balance(&a));
        assert_eq!(Ok(0), state.seq(&a));
    }
}

#[cfg(test)]
mod tests_tx {
    use ckey::{Generator, Random};
    use ctypes::errors::RuntimeError;

    use super::*;
    use crate::tests::helpers::{get_temp_state, get_test_client};
    use crate::StateError;

    fn address() -> (Address, Public) {
        let keypair = Random.generate().unwrap();
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
        check_top_level_state!(state, [(shard_text: (shard_id, Tracker::from(H256::random())))]);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn apply_create_shard() {
        let mut state = get_temp_state();
        let (sender, sender_public) = address();
        let users = vec![Address::random(), Address::random(), Address::random()];
        set_top_level_state!(state, [
            (account: users[0] => seq: 1),
            (account: users[1] => seq: 1),
            (account: users[2] => seq: 1),
            (account: sender => balance: 20)
        ]);

        let tx1 = transaction!(fee: 5, Action::CreateShard { users: vec![] });
        let tx2 = transaction!(seq: 1, fee: 5, Action::CreateShard { users: users.clone() });
        let invalid_hash = TxHash::from(H256::random());
        let signed_hash1 = TxHash::from(H256::random());
        let signed_hash2 = TxHash::from(H256::random());

        assert_eq!(Ok(None), state.shard_id_by_hash(&invalid_hash));
        assert_eq!(Ok(None), state.shard_id_by_hash(&signed_hash1));
        assert_eq!(Ok(None), state.shard_id_by_hash(&signed_hash2));

        assert_eq!(Ok(()), state.apply(&tx1, &signed_hash1, &sender_public, &get_test_client(), 0, 0, 0));

        assert_eq!(Ok(None), state.shard_id_by_hash(&invalid_hash));
        assert_eq!(Ok(Some(0)), state.shard_id_by_hash(&signed_hash1));
        assert_eq!(Ok(None), state.shard_id_by_hash(&signed_hash2));

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 20 - 5)),
            (shard: 0 => owners: [sender]),
            (shard: 1)
        ]);

        assert_eq!(Ok(()), state.apply(&tx2, &signed_hash2, &sender_public, &get_test_client(), 0, 0, 0));
        assert_eq!(Ok(None), state.shard_id_by_hash(&invalid_hash));
        assert_eq!(Ok(Some(0)), state.shard_id_by_hash(&signed_hash1));
        assert_eq!(Ok(Some(1)), state.shard_id_by_hash(&signed_hash2));

        check_top_level_state!(state, [
            (account: sender => (seq: 2, balance: 20 - 5 - 5)),
            (shard: 0 => owners: vec![sender], users: vec![]),
            (shard: 1 => owners: vec![sender], users: users),
            (shard: 2)
        ]);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn apply_create_shard_when_there_are_default_shards() {
        let mut state = get_temp_state();
        let (sender, sender_public) = address();
        let shard_owner0 = address().0;
        let shard_owner1 = address().0;
        let shard_user = Address::random();

        set_top_level_state!(state, [
            (account: shard_owner0 => seq: 1),
            (account: shard_owner1 => seq: 1),
            (account: shard_user => seq: 1),
            (shard: 0 => owners: [shard_owner0]),
            (shard: 1 => owners: [shard_owner1]),
            (metadata: shards: 2),
            (account: sender => balance: 20)
        ]);

        let tx1 = transaction!(fee: 5, Action::CreateShard { users: vec![shard_user] });
        let invalid_hash = TxHash::from(H256::random());
        let signed_hash1 = TxHash::from(H256::random());
        let signed_hash2 = TxHash::from(H256::random());

        assert_eq!(Ok(None), state.shard_id_by_hash(&invalid_hash));
        assert_eq!(Ok(None), state.shard_id_by_hash(&signed_hash1));
        assert_eq!(Ok(None), state.shard_id_by_hash(&signed_hash2));

        assert_eq!(Ok(()), state.apply(&tx1, &signed_hash1, &sender_public, &get_test_client(), 0, 0, 0));

        assert_eq!(Ok(None), state.shard_id_by_hash(&invalid_hash));
        assert_eq!(Ok(Some(2)), state.shard_id_by_hash(&signed_hash1));
        assert_eq!(Ok(None), state.shard_id_by_hash(&signed_hash2));

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 20 - 5)),
            (shard: 0 => owners: [shard_owner0]),
            (shard: 1 => owners: vec![shard_owner1]),
            (shard: 2 => owners: vec![sender], users: vec![shard_user]),
            (shard: 3)
        ]);

        let tx2 = transaction!(seq: 1, fee: 5, Action::CreateShard { users: vec![] });
        assert_eq!(Ok(()), state.apply(&tx2, &signed_hash2, &sender_public, &get_test_client(), 0, 0, 0));
        assert_eq!(Ok(None), state.shard_id_by_hash(&invalid_hash));
        assert_eq!(Ok(Some(2)), state.shard_id_by_hash(&signed_hash1));
        assert_eq!(Ok(Some(3)), state.shard_id_by_hash(&signed_hash2));

        check_top_level_state!(state, [
            (account: sender => (seq: 2, balance: 20 - 5 - 5)),
            (shard: 0 => owners: [shard_owner0]),
            (shard: 1 => owners: [shard_owner1]),
            (shard: 2 => owners: vec![sender], users: vec![shard_user]),
            (shard: 3 => owners: vec![sender], users: vec![]),
            (shard: 4)
        ]);
    }

    #[test]
    fn get_shard_text_in_invalid_shard2() {
        let mut state = get_temp_state();
        let (sender, sender_public) = address();
        set_top_level_state!(state, [
            (account: sender => balance: 20)
        ]);
        let tx = transaction!(fee: 5, Action::CreateShard { users: vec![] });
        assert_eq!(Ok(()), state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0));

        let invalid_shard_id = 3;
        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 20 - 5)),
            (shard: 0 => owners: vec![sender], users: vec![]),
            (shard_text: (invalid_shard_id, Tracker::from(H256::random())))
        ]);
    }

    #[test]
    fn set_shard_owners() {
        let (sender, sender_public) = address();

        let shard_id = 0;

        let mut state = get_temp_state();
        let owners = vec![Address::random(), Address::random(), sender];
        set_top_level_state!(state, [
            (account: sender => balance: 100),
            (account: owners[0] => balance: 100),
            (account: owners[1] => balance: 100),
            (shard: shard_id => owners: [sender]),
            (metadata: shards: 1)
        ]);

        let tx = transaction!(fee: 5, set_shard_owners!(owners.clone()));
        assert_eq!(Ok(()), state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0));

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 100 - 5)),
            (shard: 0 => owners: owners)
        ]);
    }

    #[test]
    fn new_owners_must_contain_sender() {
        let (sender, sender_public) = address();

        let shard_id = 0;

        let mut state = get_temp_state();
        let owners = vec![Address::random(), Address::random()];
        set_top_level_state!(state, [
            (account: sender => balance: 100),
            (account: owners[0] => seq: 1),
            (account: owners[0] => seq: 1),
            (shard: shard_id => owners: [sender]),
            (metadata: shards: 1)
        ]);

        let tx = transaction!(fee: 5, set_shard_owners!(owners));
        assert_eq!(
            Err(RuntimeError::NewOwnersMustContainSender.into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );
        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 100)),
            (shard: 0 => owners: [sender])
        ]);
    }

    #[test]
    fn only_owner_can_set_owners() {
        let (original_owner, ..) = address();

        let shard_id = 0;

        let mut state = get_temp_state();
        let (sender, sender_public) = address();
        set_top_level_state!(state, [
            (account: sender => balance: 100),
            (account: original_owner => seq: 1),
            (shard: shard_id => owners: [original_owner]),
            (metadata: shards: 1)
        ]);

        let owners = vec![Address::random(), Address::random(), sender];
        let tx = transaction!(fee: 5, set_shard_owners!(owners));

        assert_eq!(
            Err(RuntimeError::InsufficientPermission.into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 100)),
            (shard: 0 => owners: [original_owner])
        ]);
    }

    #[test]
    fn set_shard_owners_fail_on_invalid_shard_id() {
        let (sender, sender_public) = address();
        let shard_id = 0;

        let mut state = get_temp_state();
        set_top_level_state!(state, [
            (account: sender => balance: 100),
            (shard: shard_id => owners: [sender]),
            (metadata: shards: 1)
        ]);

        let invalid_shard_id = 0xF;
        let owners = vec![Address::random(), Address::random(), sender];
        let tx = transaction!(fee: 5, set_shard_owners!(shard_id: invalid_shard_id, owners));

        assert_eq!(
            Err(RuntimeError::InvalidShardId(invalid_shard_id).into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 100)),
            (shard: 0 => owners: [sender]),
            (shard: invalid_shard_id)
        ]);
    }

    #[test]
    fn user_cannot_set_owners() {
        let (original_owner, ..) = address();
        let (sender, sender_public) = address();
        let shard_id = 0;

        let mut state = get_temp_state();
        set_top_level_state!(state, [
            (account: sender => balance: 100),
            (account: original_owner => seq: 1),
            (shard: shard_id => owners: [original_owner], users: [sender]),
            (metadata: shards: 1)
        ]);

        let owners = vec![Address::random(), Address::random(), sender];

        let tx = transaction!(fee: 5, set_shard_owners!(owners));
        assert_eq!(
            Err(StateError::Runtime(RuntimeError::InsufficientPermission)),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 100)),
            (shard: 0 => owners: [original_owner])
        ]);
    }

    #[test]
    fn set_shard_users() {
        let (sender, sender_public) = address();
        let old_users = vec![Address::random(), Address::random(), Address::random()];
        let shard_id = 0;

        let mut state = get_temp_state();
        let new_users = vec![Address::random(), Address::random(), sender];
        set_top_level_state!(state, [
            (account: sender => balance: 100),
            (account: old_users[0] => seq: 1),
            (account: old_users[1] => seq: 1),
            (account: old_users[2] => seq: 1),
            (account: new_users[0] => seq: 1),
            (account: new_users[1] => seq: 1),
            (shard: shard_id => owners: [sender], users: old_users),
            (metadata: shards: 1)
        ]);

        let tx = transaction!(fee: 5, set_shard_users!(new_users));

        assert_eq!(Ok(()), state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0));
        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 100 - 5))
        ]);
    }

    #[test]
    fn user_cannot_set_shard_users() {
        let (sender, sender_public) = address();
        let owners = vec![Address::random(), Address::random(), Address::random()];
        let old_users = vec![Address::random(), Address::random(), Address::random(), sender];
        let shard_id = 0;

        let mut state = get_temp_state();
        set_top_level_state!(state, [
            (account: sender => balance: 100),
            (account: old_users[0] => seq: 1),
            (account: old_users[1] => seq: 1),
            (account: old_users[2] => seq: 1),
            (account: owners[0] => seq: 1),
            (account: owners[1] => seq: 1),
            (account: owners[2] => seq: 1),
            (shard: shard_id => owners: owners.clone(), users: old_users.clone()),
            (metadata: shards: 1)
        ]);

        let new_users = vec![Address::random(), Address::random(), sender];
        let tx = transaction!(fee: 5, set_shard_users!(new_users));

        assert_eq!(
            Err(RuntimeError::InsufficientPermission.into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );
        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 100)),
            (shard: 0 => owners: owners, users: old_users)
        ]);
    }
}
