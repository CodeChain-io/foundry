// Copyright 2018-2019 Kodebox, Inc.
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

use std::cell::{RefCell, RefMut};

use cdb::AsHashDB;
use ckey::{public_to_address, verify_address, Address, Public, Signature};
use cmerkle::{Result as TrieResult, TrieError, TrieFactory};
use ctypes::errors::RuntimeError;
use ctypes::transaction::{Action, Transaction};
use ctypes::util::unexpected::Mismatch;
use ctypes::{BlockNumber, CommonParams, TxHash};
use kvdb::DBTransaction;
use primitives::{Bytes, H256};
use util_error::UtilError;

use crate::cache::TopCache;
use crate::checkpoint::{CheckpointId, StateWithCheckpoint};
use crate::traits::{StateWithCache, TopState, TopStateView};
use crate::{
    Account, ActionData, FindActionHandler, Metadata, MetadataAddress, RegularAccount, RegularAccountAddress, StateDB,
    StateResult, Text,
};

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

    fn regular_account_by_address(&self, a: &Address) -> TrieResult<Option<RegularAccount>> {
        let a = RegularAccountAddress::from_address(a);
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        Ok(self.top_cache.regular_account(&a, &trie)?)
    }

    fn metadata(&self) -> TrieResult<Option<Metadata>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let address = MetadataAddress::new();
        self.top_cache.metadata(&address, &trie)
    }

    fn text(&self, key: &H256) -> TrieResult<Option<Text>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        Ok(self.top_cache.text(key, &trie)?.map(Into::into))
    }

    fn action_data(&self, key: &H256) -> TrieResult<Option<ActionData>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        Ok(self.top_cache.action_data(key, &trie)?.map(Into::into))
    }
}

impl StateWithCache for TopLevelState {
    fn commit(&mut self) -> StateResult<H256> {
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
    }

    fn discard_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is discarded", id);
        self.top_cache.discard_checkpoint();
    }

    fn revert_to_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for top level is reverted", id);
        self.top_cache.revert_to_checkpoint();
    }
}

impl TopLevelState {
    /// Creates new state with existing state root
    pub fn from_existing(db: StateDB, root: H256) -> Result<Self, TrieError> {
        if !db.as_hashdb().contains(&root) {
            return Err(TrieError::InvalidStateRoot(root))
        }

        let top_cache = db.top_cache();

        let state = TopLevelState {
            db: RefCell::new(db),
            root,
            top_cache,
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
        signer_public: &Public,
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
        current_block_timestamp: u64,
    ) -> StateResult<()> {
        self.create_checkpoint(FEE_CHECKPOINT);
        let result = self.apply_internal(
            tx,
            signed_hash,
            signer_public,
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

    fn apply_internal<C: FindActionHandler>(
        &mut self,
        tx: &Transaction,
        signed_hash: &TxHash,
        signer_public: &Public,
        client: &C,
        _parent_block_number: BlockNumber,
        _parent_block_timestamp: u64,
        _current_block_timestamp: u64,
    ) -> StateResult<()> {
        let fee_payer = if self.regular_account_exists_and_not_null(signer_public)? {
            let regular_account = self.get_regular_account_mut(signer_public)?;
            public_to_address(&regular_account.owner_public())
        } else {
            let address = public_to_address(signer_public);

            if !tx.is_master_key_allowed() {
                let account = self.get_account_mut(&address)?;
                if account.regular_key().is_some() {
                    return Err(RuntimeError::CannotUseMasterKey.into())
                }
            }
            address
        };
        let seq = self.seq(&fee_payer)?;

        if tx.seq != seq {
            return Err(RuntimeError::InvalidSeq(Mismatch {
                expected: seq,
                found: tx.seq,
            })
            .into())
        }

        let fee = tx.fee;

        self.inc_seq(&fee_payer)?;
        self.sub_balance(&fee_payer, fee)?;

        // The failed transaction also must pay the fee and increase seq.
        self.create_checkpoint(ACTION_CHECKPOINT);
        let result = self.apply_action(&tx.action, signed_hash, &fee_payer, signer_public, client);
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
    fn apply_action<C: FindActionHandler>(
        &mut self,
        action: &Action,
        signed_hash: &TxHash,
        fee_payer: &Address,
        signer_public: &Public,
        client: &C,
    ) -> StateResult<()> {
        match action {
            Action::Pay {
                receiver,
                quantity,
            } => {
                self.transfer_balance(fee_payer, receiver, *quantity)?;
                Ok(())
            }
            Action::SetRegularKey {
                key,
            } => {
                self.set_regular_key(signer_public, key)?;
                Ok(())
            }
            Action::Store {
                content,
                certifier,
                signature,
            } => {
                let text = Text::new(content, certifier);
                self.store_text(signed_hash, text, signature)?;
                Ok(())
            }
            Action::Remove {
                hash,
                signature,
            } => {
                self.remove_text(hash, signature)?;
                Ok(())
            }
            Action::Custom {
                handler_id,
                bytes,
            } => {
                let handler = client.find_action_handler_for(*handler_id).expect("Unknown custom parsel applied!");
                handler.execute(bytes, self, fee_payer, signer_public)?;
                Ok(())
            }
        }
    }

    fn get_account_mut(&self, a: &Address) -> TrieResult<RefMut<Account>> {
        debug_assert_eq!(Ok(false), self.regular_account_exists_and_not_null_by_address(a));

        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.account_mut(&a, &trie)
    }

    fn get_regular_account_mut(&self, public: &Public) -> TrieResult<RefMut<RegularAccount>> {
        let regular_account_address = RegularAccountAddress::new(public);
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.regular_account_mut(&regular_account_address, &trie)
    }

    fn get_metadata_mut(&self) -> TrieResult<RefMut<Metadata>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        let address = MetadataAddress::new();
        self.top_cache.metadata_mut(&address, &trie)
    }

    fn get_text(&self, key: &TxHash) -> TrieResult<Option<Text>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.text(key, &trie)
    }

    fn get_text_mut(&self, key: &TxHash) -> TrieResult<RefMut<Text>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.text_mut(key, &trie)
    }

    fn get_action_data_mut(&self, key: &H256) -> TrieResult<RefMut<ActionData>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.top_cache.action_data_mut(key, &trie)
    }

    pub fn journal_under(&self, batch: &mut DBTransaction, now: u64) -> Result<u32, UtilError> {
        self.db.borrow_mut().journal_under(batch, now, self.root)
    }

    pub fn top_cache(&self) -> &TopCache {
        &self.top_cache
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
        }
    }
}

impl TopState for TopLevelState {
    fn kill_account(&mut self, account: &Address) {
        self.top_cache.remove_account(account);
    }

    fn kill_regular_account(&mut self, account: &Public) {
        self.top_cache.remove_regular_account(&RegularAccountAddress::new(account));
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
        if self.regular_account_exists_and_not_null_by_address(to)? {
            return Err(RuntimeError::InvalidTransferDestination.into())
        }
        self.sub_balance(from, by)?;
        self.add_balance(to, by)?;
        Ok(())
    }

    fn inc_seq(&mut self, a: &Address) -> TrieResult<()> {
        self.get_account_mut(a)?.inc_seq();
        Ok(())
    }

    fn set_regular_key(&mut self, signer_public: &Public, regular_key: &Public) -> StateResult<()> {
        let (owner_public, owner_address) = if self.regular_account_exists_and_not_null(signer_public)? {
            let regular_account = self.get_regular_account_mut(&signer_public)?;
            let owner_public = regular_account.owner_public();
            let owner_address = public_to_address(owner_public);
            (*owner_public, owner_address)
        } else {
            (*signer_public, public_to_address(&signer_public))
        };

        if self.regular_account_exists_and_not_null(regular_key)? {
            return Err(RuntimeError::RegularKeyAlreadyInUse.into())
        }

        let regular_address = public_to_address(regular_key);
        if self.account_exists_and_not_null(&regular_address)? {
            return Err(RuntimeError::RegularKeyAlreadyInUseAsPlatformAccount.into())
        }

        let prev_regular_key = self.get_account_mut(&owner_address)?.regular_key();

        if let Some(prev_regular_key) = prev_regular_key {
            self.kill_regular_account(&prev_regular_key);
        }

        let mut owner_account = self.get_account_mut(&owner_address)?;
        owner_account.set_regular_key(regular_key);
        self.get_regular_account_mut(&regular_key)?.set_owner_public(&owner_public);
        Ok(())
    }

    fn store_text(&mut self, key: &TxHash, text: Text, sig: &Signature) -> StateResult<()> {
        match verify_address(text.certifier(), sig, &text.content_hash()) {
            Ok(false) => {
                return Err(RuntimeError::TextVerificationFail("Certifier and signer are different".to_string()).into())
            }
            Err(err) => return Err(RuntimeError::TextVerificationFail(err.to_string()).into()),
            _ => {}
        }
        let mut text_entry = self.get_text_mut(key)?;
        *text_entry = text;
        Ok(())
    }

    fn remove_text(&mut self, key: &TxHash, sig: &Signature) -> StateResult<()> {
        let text = self.get_text(key)?.ok_or_else(|| RuntimeError::TextNotExist)?;
        match verify_address(text.certifier(), sig, key) {
            Ok(false) => {
                return Err(RuntimeError::TextVerificationFail("Certifier and signer are different".to_string()).into())
            }
            Err(err) => return Err(RuntimeError::TextVerificationFail(err.to_string()).into()),
            _ => {}
        }
        self.top_cache.remove_text(key);
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
}

#[cfg(test)]
mod tests_state {
    use std::sync::Arc;

    use ccrypto::BLAKE_NULL_RLP;
    use cdb::{new_journaldb, Algorithm};

    use super::*;
    use crate::tests::helpers::{empty_top_state, get_memory_db, get_temp_state, get_temp_state_db};

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
            let mut state = empty_top_state(StateDB::new(jorunal));
            assert_eq!(Ok(()), state.inc_seq(&a));
            assert_eq!(Ok(()), state.add_balance(&a, 100));
            assert_eq!(Ok(100), state.balance(&a));
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(100), state.balance(&a));

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(1, records.unwrap());
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
            let mut state = empty_top_state(StateDB::new(jorunal));
            assert_eq!(Ok(()), state.inc_seq(&a));
            assert_eq!(Ok(()), state.add_balance(&a, 69));
            assert_eq!(Ok(69), state.balance(&a));
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(69), state.balance(&a));

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(1, records.unwrap());
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
            let mut state = empty_top_state(StateDB::new(jorunal));
            assert_eq!(Ok(()), state.inc_seq(&a));
            let root = state.commit();
            assert!(root.is_ok(), "{:?}", root);
            assert_eq!(Ok(true), state.account_exists(&a));
            assert_eq!(Ok(1), state.seq(&a));

            let mut transaction = memory_db.transaction();
            let records = state.journal_under(&mut transaction, 1);
            assert!(records.is_ok(), "{:?}", records);
            assert_eq!(1, records.unwrap());
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
            assert_eq!(0, records.unwrap());
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
        assert_eq!(Ok(H256::from("db4046bb91a12a37cbfb0f09631aad96a97248423163eca791e19b430cc7fe4a")), state.commit());
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

    #[test]
    fn create_empty() {
        let mut state = get_temp_state();
        assert_eq!(Ok(BLAKE_NULL_RLP), state.commit());
    }
}

#[cfg(test)]
mod tests_tx {
    use ccrypto::Blake;
    use ckey::{sign, Generator, Private, Random};
    use ctypes::errors::RuntimeError;

    use rlp::Encodable;

    use super::*;
    use crate::tests::helpers::{get_temp_state, get_test_client};
    use crate::StateError;

    fn address() -> (Address, Public, Private) {
        let keypair = Random.generate().unwrap();
        (keypair.address(), *keypair.public(), *keypair.private())
    }

    #[test]
    fn apply_error_for_invalid_seq() {
        let mut state = get_temp_state();

        let (sender, sender_public, _) = address();
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

        let (sender, sender_public, _) = address();
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

        let (sender, sender_public, _) = address();
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
    fn apply_set_regular_key() {
        let mut state = get_temp_state();
        let key = 1u64.into();

        let (sender, sender_public, _) = address();
        set_top_level_state!(state, [(account: sender => balance: 5)]);

        let tx = transaction!(fee: 5, set_regular_key!(key));
        assert_eq!(Ok(()), state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0));

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 0, key: key))
        ]);
    }

    #[test]
    fn cannot_pay_to_regular_account() {
        let mut state = get_temp_state();

        let (sender, sender_public, _) = address();
        let (master_account, master_public, _) = address();
        let (regular_account, regular_public, _) = address();
        set_top_level_state!(state, [
            (account: sender => balance: 123),
            (account: master_account => balance: 456),
            (regular_key: master_public => regular_public)
        ]);

        let tx = transaction!(fee: 5, pay!(regular_account, 10));
        assert_eq!(
            Err(RuntimeError::InvalidTransferDestination.into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 123)),
            (account: master_account => (seq: 0, balance: 456, key: regular_public))
        ]);
    }

    #[test]
    fn use_owner_balance_when_signed_with_regular_key() {
        let mut state = get_temp_state();

        let (sender, sender_public, _) = address();
        set_top_level_state!(state, [(account: sender => balance: 15)]);

        let regular_keypair = Random.generate().unwrap();
        let key = regular_keypair.public();
        let tx = transaction!(fee: 5, set_regular_key!(*key));

        assert_eq!(Ok(()), state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0));

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 10, key: *key))
        ]);
    }

    #[test]
    fn fail_when_two_accounts_used_the_same_regular_key() {
        let mut state = get_temp_state();

        let (sender, sender_public, _) = address();
        let (sender2, sender_public2, _) = address();
        set_top_level_state!(state, [
            (account: sender => balance: 15),
            (account: sender2 => balance: 15)
        ]);

        let regular_keypair = Random.generate().unwrap();
        let key = regular_keypair.public();
        let tx = transaction!(fee: 5, set_regular_key!(*key));

        assert_eq!(Ok(()), state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0));

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 10, key: *key)),
            (account: sender2 => (seq: 0, balance: 15, key))
        ]);

        let tx = transaction!(fee: 5, set_regular_key!(*key));
        assert_eq!(
            Err(RuntimeError::RegularKeyAlreadyInUse.into()),
            state.apply(&tx, &H256::random().into(), &sender_public2, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 10, key: *key)),
            (account: sender2 => (seq: 0, balance: 15, key))
        ]);
    }

    #[test]
    fn fail_when_regular_key_is_already_registered_as_owner_key() {
        let mut state = get_temp_state();

        let (sender, sender_public, _) = address();
        let (sender2, sender_public2, _) = address();
        set_top_level_state!(state, [
            (account: sender => balance: 20),
            (account: sender2 => balance: 20)
        ]);

        let tx = transaction! (fee: 5, set_regular_key!(sender_public2));
        assert_eq!(
            Err(RuntimeError::RegularKeyAlreadyInUseAsPlatformAccount.into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 20))
        ]);
    }

    #[test]
    fn change_regular_key() {
        let mut state = get_temp_state();

        let (sender, sender_public, _) = address();
        let (_, regular_public, _) = address();
        set_top_level_state!(state, [
            (account: sender => balance: 20),
            (regular_key: sender_public => regular_public)
        ]);

        assert_eq!(Ok(true), state.regular_account_exists_and_not_null(&regular_public));

        let (_, regular_public2, _) = address();
        let tx = transaction! (fee: 5, set_regular_key!(regular_public2));
        assert_eq!(Ok(()), state.apply(&tx, &H256::random().into(), &regular_public, &get_test_client(), 0, 0, 0));

        assert_eq!(Ok(false), state.regular_account_exists_and_not_null(&regular_public));
        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 20 - 5, key: regular_public2))
        ]);
    }

    #[test]
    fn fail_when_someone_sends_some_ccc_to_an_address_which_used_as_a_regular_key() {
        let (sender, sender_public, _) = address();
        let (receiver, receiver_public, _) = address();
        let (regular_address, regular_public, _) = address();

        let mut state = get_temp_state();
        set_top_level_state!(state, [
            (account: sender => balance: 20),
            (regular_key: receiver_public => regular_public)
        ]);

        let tx = transaction!(fee: 5, pay!(regular_address, 5));
        assert_eq!(
            Err(RuntimeError::InvalidTransferDestination.into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 20)),
            (account: receiver => (seq: 0, balance: 0, key: regular_public))
        ]);
    }

    #[test]
    fn fail_when_tried_to_use_master_key_instead_of_regular_key() {
        let (sender, sender_public, _) = address();
        let (_, regular_public, _) = address();
        let (receiver_address, ..) = address();

        let mut state = get_temp_state();
        set_top_level_state!(state, [
            (account: sender => balance: 20),
            (regular_key: sender_public => regular_public)
        ]);

        let tx = transaction!(fee: 5, pay!(receiver_address, 5));
        assert_eq!(
            Err(RuntimeError::CannotUseMasterKey.into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 20, key: regular_public)),
            (account: receiver_address => (seq: 0, balance: 0))
        ]);
    }

    #[test]
    fn apply_error_for_action_failure() {
        let mut state = get_temp_state();
        let (sender, sender_public, _) = address();
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
    fn store_and_remove() {
        let (sender, sender_public, sender_private) = address();

        let mut state = get_temp_state();
        set_top_level_state!(state, [
            (account: sender => balance: 20)
        ]);

        let content = "CodeChain".to_string();
        let content_hash = Blake::blake(content.rlp_bytes());
        let signature = sign(&sender_private, &content_hash).unwrap();

        let store_tx = transaction!(fee: 10, store!(content.clone(), sender, signature));
        let dummy_signed_hash = TxHash::from(H256::random());

        assert_eq!(Ok(()), state.apply(&store_tx, &dummy_signed_hash, &sender_public, &get_test_client(), 0, 0, 0));

        check_top_level_state!(state, [
            (account: sender => (seq: 1, balance: 10)),
            (text: &dummy_signed_hash => { content: &content, certifier: &sender })
        ]);

        let signature = sign(&sender_private, &dummy_signed_hash).unwrap();
        let remove_tx = transaction!(seq: 1, fee: 10, remove!(dummy_signed_hash, signature));

        assert_eq!(
            Ok(()),
            state.apply(&remove_tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 2, balance: 0)),
            (text: &dummy_signed_hash)
        ]);
    }

    #[test]
    fn store_with_wrong_signature() {
        let (sender, sender_public, _) = address();

        let mut state = get_temp_state();
        set_top_level_state!(state, [
            (account: sender => balance: 20)
        ]);

        let content = "CodeChain".to_string();
        let content_hash = Blake::blake(content.rlp_bytes());
        let signature = Signature::random();

        let tx = transaction!(fee: 10, store!(content.clone(), sender, signature));

        match state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0) {
            Err(StateError::Runtime(RuntimeError::TextVerificationFail(_))) => {}
            err => panic!("The transaction must fail with text verification failure, but {:?}", err),
        }

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 20)),
            (text: &tx.hash())
        ]);

        let signature = sign(Random.generate().unwrap().private(), &content_hash).unwrap();

        let tx = transaction!(seq: 0, fee: 10, store!(content, sender, signature));

        assert_eq!(
            Err(RuntimeError::TextVerificationFail("Certifier and signer are different".to_string()).into()),
            state.apply(&tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 20)),
            (text: &tx.hash())
        ]);
    }

    #[test]
    fn remove_on_nothing() {
        let (sender, sender_public, sender_private) = address();

        let mut state = get_temp_state();
        set_top_level_state!(state, [
            (account: sender => balance: 20)
        ]);

        let hash = TxHash::from(H256::random());
        let signature = sign(&sender_private, &hash).unwrap();
        let remove_tx = transaction!(fee: 10, remove!(hash, signature));

        assert_eq!(
            Err(RuntimeError::TextNotExist.into()),
            state.apply(&remove_tx, &H256::random().into(), &sender_public, &get_test_client(), 0, 0, 0)
        );

        check_top_level_state!(state, [
            (account: sender => (seq: 0, balance: 20))
        ]);
    }
}
