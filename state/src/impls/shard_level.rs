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

use crate::cache::ShardCache;
use crate::checkpoint::{CheckpointId, StateWithCheckpoint};
use crate::traits::{ShardState, ShardStateView};
use crate::{
    Asset, AssetScheme, AssetSchemeAddress, OwnedAsset, OwnedAssetAddress, ShardText, ShardTextAddress, StateDB,
    StateResult,
};
use ccrypto::BLAKE_NULL_RLP;
use cdb::AsHashDB;
use ckey::Address;
use ctypes::transaction::ShardTransaction;
use ctypes::{BlockNumber, ShardId, Tracker};
use cvm::ChainTimeInfo;
use merkle_trie::{Result as TrieResult, TrieError, TrieFactory};
use primitives::{Bytes, H160, H256};
use std::cell::RefCell;

pub struct ShardLevelState<'db> {
    db: &'db mut RefCell<StateDB>,
    root: H256,
    cache: &'db mut ShardCache,
    id_of_checkpoints: Vec<CheckpointId>,
    shard_id: ShardId,
}

impl<'db> ShardLevelState<'db> {
    /// Creates new state with empty state root
    pub fn try_new(shard_id: ShardId, db: &'db mut RefCell<StateDB>, cache: &'db mut ShardCache) -> StateResult<Self> {
        let root = BLAKE_NULL_RLP;
        Ok(Self {
            db,
            root,
            cache,
            id_of_checkpoints: Default::default(),
            shard_id,
        })
    }

    /// Creates new state with existing state root
    pub fn from_existing(
        shard_id: ShardId,
        db: &'db mut RefCell<StateDB>,
        root: H256,
        cache: &'db mut ShardCache,
    ) -> TrieResult<Self> {
        if !db.borrow().as_hashdb().contains(&root) {
            return Err(TrieError::InvalidStateRoot(root))
        }

        Ok(Self {
            db,
            root,
            cache,
            id_of_checkpoints: Default::default(),
            shard_id,
        })
    }

    /// Creates immutable shard state
    pub fn read_only(
        shard_id: ShardId,
        db: &RefCell<StateDB>,
        root: H256,
        cache: ShardCache,
    ) -> TrieResult<ReadOnlyShardLevelState<'_>> {
        if !db.borrow().as_hashdb().contains(&root) {
            return Err(TrieError::InvalidStateRoot(root))
        }

        Ok(ReadOnlyShardLevelState {
            db,
            root,
            cache,
            shard_id,
        })
    }

    fn apply_internal<C: ChainTimeInfo>(
        &mut self,
        transaction: &ShardTransaction,
        _sender: &Address,
        _shard_users: &[Address],
        _approvers: &[Address],
        _client: &C,
        _parent_block_number: BlockNumber,
        _parent_block_timestamp: u64,
    ) -> StateResult<()> {
        match transaction {
            ShardTransaction::ShardStore {
                shard_id,
                content,
                ..
            } => {
                assert_eq!(*shard_id, self.shard_id);
                self.store_text(transaction.tracker(), content.to_string())
            }
            ShardTransaction::MintAsset {
                ..
            } => panic!("To be removed"),
            ShardTransaction::TransferAsset {
                ..
            } => panic!("To be removed"),
            ShardTransaction::ChangeAssetScheme {
                ..
            } => panic!("To be removed"),
            ShardTransaction::IncreaseAssetSupply {
                ..
            } => panic!("To be removed"),
            ShardTransaction::UnwrapCCC {
                ..
            } => panic!("To be removed"),
            ShardTransaction::WrapCCC {
                ..
            } => panic!("To be removed"),
        }
    }

    pub fn create_asset_scheme(
        &self,
        shard_id: ShardId,
        asset_type: H160,
        metadata: String,
        supply: u64,
        approver: Option<Address>,
        registrar: Option<Address>,
        allowed_script_hashes: Vec<H160>,
        pool: Vec<Asset>,
    ) -> TrieResult<AssetScheme> {
        self.cache.create_asset_scheme(&AssetSchemeAddress::new(asset_type, shard_id), || {
            AssetScheme::new_with_pool(metadata, supply, approver, registrar, allowed_script_hashes, pool)
        })
    }

    pub fn create_asset(
        &self,
        tracker: Tracker,
        index: usize,
        asset_type: H160,
        lock_script_hash: H160,
        parameters: Vec<Bytes>,
        quantity: u64,
    ) -> TrieResult<OwnedAsset> {
        self.cache.create_asset(&OwnedAssetAddress::new(tracker, index, self.shard_id), || {
            OwnedAsset::new(asset_type, lock_script_hash, parameters, quantity)
        })
    }

    fn store_text(&self, tracker: Tracker, content: String) -> StateResult<()> {
        self.cache.create_shard_text(&ShardTextAddress::new(tracker, self.shard_id), || ShardText::new(&content))?;
        Ok(())
    }

    #[cfg(test)]
    fn shard_id(&self) -> ShardId {
        self.shard_id
    }
}

impl<'db> ShardStateView for ShardLevelState<'db> {
    fn asset_scheme(&self, asset_type: H160) -> TrieResult<Option<AssetScheme>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.asset_scheme(&AssetSchemeAddress::new(asset_type, self.shard_id), &trie)
    }

    fn asset(&self, tracker: Tracker, index: usize) -> Result<Option<OwnedAsset>, TrieError> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.asset(&OwnedAssetAddress::new(tracker, index, self.shard_id), &trie)
    }

    fn text(&self, tracker: Tracker) -> Result<Option<ShardText>, TrieError> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.shard_text(&ShardTextAddress::new(tracker, self.shard_id), &trie)
    }
}

impl<'db> StateWithCheckpoint for ShardLevelState<'db> {
    fn create_checkpoint(&mut self, id: CheckpointId) {
        ctrace!(STATE, "Checkpoint({}) for shard({}) is created", id, self.shard_id);
        self.id_of_checkpoints.push(id);
        self.cache.checkpoint();
    }

    fn discard_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for shard({}) is discarded", id, self.shard_id);
        self.cache.discard_checkpoint();
    }

    fn revert_to_checkpoint(&mut self, id: CheckpointId) {
        let expected = self.id_of_checkpoints.pop().expect("The checkpoint must exist");
        assert_eq!(expected, id);

        ctrace!(STATE, "Checkpoint({}) for shard({}) is reverted", id, self.shard_id);
        self.cache.revert_to_checkpoint();
    }
}

const TRANSACTION_CHECKPOINT: CheckpointId = 456;

impl<'db> ShardState for ShardLevelState<'db> {
    fn apply<C: ChainTimeInfo>(
        &mut self,
        transaction: &ShardTransaction,
        sender: &Address,
        shard_users: &[Address],
        approvers: &[Address],
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
    ) -> StateResult<()> {
        ctrace!(TX, "Execute InnerTx {:?}(InnerTxHash:{:?})", transaction, transaction.tracker());

        self.create_checkpoint(TRANSACTION_CHECKPOINT);
        let result = self.apply_internal(
            transaction,
            sender,
            shard_users,
            approvers,
            client,
            parent_block_number,
            parent_block_timestamp,
        );
        match result {
            Ok(_) => {
                self.discard_checkpoint(TRANSACTION_CHECKPOINT);
                Ok(())
            }
            Err(err) => {
                self.revert_to_checkpoint(TRANSACTION_CHECKPOINT);
                Err(err)
            }
        }
    }
}

pub struct ReadOnlyShardLevelState<'db> {
    db: &'db RefCell<StateDB>,
    root: H256,
    cache: ShardCache,
    shard_id: ShardId,
}

impl<'db> ShardStateView for ReadOnlyShardLevelState<'db> {
    fn asset_scheme(&self, asset_type: H160) -> TrieResult<Option<AssetScheme>> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.asset_scheme(&AssetSchemeAddress::new(asset_type, self.shard_id), &trie)
    }

    fn asset(&self, tracker: Tracker, index: usize) -> Result<Option<OwnedAsset>, TrieError> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.asset(&OwnedAssetAddress::new(tracker, index, self.shard_id), &trie)
    }

    fn text(&self, tracker: Tracker) -> Result<Option<ShardText>, TrieError> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.shard_text(&ShardTextAddress::new(tracker, self.shard_id), &trie)
    }
}

#[cfg(test)]
mod tests {
    use ctypes::TxHash;

    use super::super::super::StateError;
    use super::super::test_helper::SHARD_ID;
    use super::*;
    use crate::tests::helpers::{get_temp_state_db, get_test_client};

    fn address() -> Address {
        Address::random()
    }

    fn get_temp_shard_state<'d>(
        state_db: &'d mut RefCell<StateDB>,
        shard_id: ShardId,
        cache: &'d mut ShardCache,
    ) -> ShardLevelState<'d> {
        ShardLevelState::try_new(shard_id, state_db, cache).unwrap()
    }

    #[test]
    fn mint_permissioned_asset() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let amount = 100;
        let approver = Address::random();
        let transaction =
            asset_mint!(asset_mint_output!(lock_script_hash, parameters, amount), metadata.clone(), approver: approver);

        let transaction_tracker = transaction.tracker();
        let asset_type = Blake::blake(*transaction_tracker);
        assert_eq!(Ok(()), state.apply(&transaction, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, approver: approver }),
            (asset: (transaction_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);
    }

    #[test]
    fn mint_infinite_asset() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let approver = Address::random();
        let transaction = asset_mint!(
            asset_mint_output!(lock_script_hash, parameters: parameters),
            metadata.clone(),
            approver: approver
        );
        let transaction_tracker = transaction.tracker();
        let asset_type = Blake::blake(*transaction_tracker);

        assert_eq!(Ok(()), state.apply(&transaction, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: ::std::u64::MAX, approver: approver }),
            (asset: (transaction_tracker, 0) => { asset_type: asset_type, quantity: ::std::u64::MAX })
        ]);
    }

    #[test]
    fn cannot_mint_twice() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let approver = Address::random();
        let transaction = asset_mint!(
            asset_mint_output!(lock_script_hash, parameters: parameters),
            metadata.clone(),
            approver: approver
        );

        let transaction_tracker = transaction.tracker();
        let asset_type = Blake::blake(*transaction_tracker);
        assert_eq!(Ok(()), state.apply(&transaction, &sender, &[sender], &[], &get_test_client(), 0, 0));

        assert_eq!(
            Err(StateError::Runtime(RuntimeError::AssetSchemeDuplicated {
                tracker: transaction_tracker,
                shard_id: SHARD_ID
            })),
            state.apply(&transaction, &sender, &[sender], &[], &get_test_client(), 0, 0)
        );

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: ::std::u64::MAX, approver: approver }),
            (asset: (transaction_tracker, 0) => { asset_type: asset_type, quantity: ::std::u64::MAX })
        ]);
    }

    #[test]
    fn invalid_approver() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let approver = Address::random();
        let amount = 30;
        let mint =
            asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata.clone(), approver: approver);
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, approver: approver }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 30), vec![0x30, 0x01])],
            asset_transfer_outputs![(lock_script_hash, asset_type, 30)]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(
            Err(StateError::Runtime(RuntimeError::NotApproved(approver))),
            state.apply(&transfer, &sender, &[sender], &[], &get_test_client(), 0, 0)
        );

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, approver: approver }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount }),
            (asset: (transfer_tracker, 0))
        ]);
    }

    #[test]
    fn mint_and_transfer() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let amount = 30;
        let mint = asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata.clone());
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let random_lock_script_hash = H160::random();
        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 30), vec![0x30, 0x01])],
            asset_transfer_outputs![
                (lock_script_hash, vec![vec![1]], asset_type, 10),
                (lock_script_hash, asset_type, 5),
                (random_lock_script_hash, asset_type, 15),
            ]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(Ok(()), state.apply(&transfer, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount }),
            (asset: (mint_tracker, 0)),
            (asset: (transfer_tracker, 0) => { asset_type: asset_type, quantity: 10, lock_script_hash: lock_script_hash }),
            (asset: (transfer_tracker, 1) => { asset_type: asset_type, quantity: 5, lock_script_hash: lock_script_hash }),
            (asset: (transfer_tracker, 2) => { asset_type: asset_type, quantity: 15, lock_script_hash: random_lock_script_hash })
        ]);
    }

    #[test]
    fn mint_and_transfer_allowed() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let random_lock_script_hash = H160::random();
        let allowed_script_hashes = vec![lock_script_hash, random_lock_script_hash];

        let amount = 30;
        let mint = asset_mint!(
            asset_mint_output!(lock_script_hash, supply: amount),
            metadata.clone(),
            allowed_script_hashes: allowed_script_hashes.clone()
        );
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, allowed_script_hashes: allowed_script_hashes}),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 30), vec![0x30, 0x01])],
            asset_transfer_outputs![
                (lock_script_hash, vec![vec![1]], asset_type, 10),
                (lock_script_hash, asset_type, 5),
                (random_lock_script_hash, asset_type, 15),
            ]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(Ok(()), state.apply(&transfer, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, allowed_script_hashes: allowed_script_hashes}),
            (asset: (mint_tracker, 0)),
            (asset: (transfer_tracker, 0) => { asset_type: asset_type, quantity: 10, lock_script_hash: lock_script_hash }),
            (asset: (transfer_tracker, 1) => { asset_type: asset_type, quantity: 5, lock_script_hash: lock_script_hash }),
            (asset: (transfer_tracker, 2) => { asset_type: asset_type, quantity: 15, lock_script_hash: random_lock_script_hash })
        ]);
    }

    #[test]
    fn mint_and_transfer_not_allowed() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let amount = 30;
        let allowed_lock_script_hash = H160::from("ca5d3fa0a6887285ef6aa85cb12960a2b6706e00");
        let allowed_script_hashes = vec![allowed_lock_script_hash];
        let mint = asset_mint!(
            asset_mint_output!(lock_script_hash, supply: amount),
            metadata.clone(),
            allowed_script_hashes: allowed_script_hashes.clone()
        );
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, allowed_script_hashes: allowed_script_hashes}),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 30), vec![0x30, 0x01])],
            asset_transfer_outputs![(lock_script_hash, asset_type, 30)]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(
            Err(StateError::Runtime(RuntimeError::ScriptNotAllowed(lock_script_hash))),
            state.apply(&transfer, &sender, &[sender], &[], &get_test_client(), 0, 0)
        );

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, allowed_script_hashes: allowed_script_hashes}),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount }),
            (asset: (transfer_tracker, 0))
        ]);
    }

    #[test]
    fn mint_and_burn() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("ca5d3fa0a6887285ef6aa85cb12960a2b6706e00");
        let amount = 30;
        let mint = asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata.clone());
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let burn = asset_transfer!(
            burns: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, amount), vec![0x01])]
        );

        let burn_tracker = burn.tracker();

        assert_eq!(Ok(()), state.apply(&burn, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: 0 }),
            (asset: (mint_tracker, 0)),
            (asset: (burn_tracker, 0))
        ]);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn mint_and_transfer_and_burn() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let amount = 30;
        let mint = asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata.clone());
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let burn_amount = 5;
        let lock_script_hash_burn = H160::from("ca5d3fa0a6887285ef6aa85cb12960a2b6706e00");
        let random_lock_script_hash = H160::random();
        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 30), vec![0x30, 0x01])],
            asset_transfer_outputs![
                (lock_script_hash, vec![vec![1]], asset_type, 10),
                (lock_script_hash_burn, asset_type, burn_amount),
                (random_lock_script_hash, asset_type, 15),
            ]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(Ok(()), state.apply(&transfer, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount }),
            (asset: (mint_tracker, 0)),
            (asset: (transfer_tracker, 0) => { asset_type: asset_type, quantity: 10 }),
            (asset: (transfer_tracker, 1) => { asset_type: asset_type, quantity: burn_amount }),
            (asset: (transfer_tracker, 2) => { asset_type: asset_type, quantity: 15 })
        ]);

        let burn = asset_transfer!(
            burns: asset_transfer_inputs![(asset_out_point!(transfer_tracker, 1, asset_type, 5), vec![0x01])]
        );
        let burn_tracker = burn.tracker();

        assert_eq!(Ok(()), state.apply(&burn, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount - burn_amount }),
            (asset: (mint_tracker, 0)),
            (asset: (transfer_tracker, 0) => { asset_type: asset_type, quantity: 10 }),
            (asset: (transfer_tracker, 1)),
            (asset: (transfer_tracker, 2) => { asset_type: asset_type, quantity: 15 }),
            (asset: (burn_tracker, 0))
        ]);
    }

    #[test]
    fn registrar_can_transfer() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let registrar = address();
        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let amount = 30;
        let mint =
            asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata.clone(), registrar: registrar);
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, registrar: registrar }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let lock_script_hash1 = H160::random();
        let lock_script_hash2 = H160::random();
        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 30))],
            asset_transfer_outputs![
                (lock_script_hash, vec![vec![1]], asset_type, 10),
                (lock_script_hash1, asset_type, 5),
                (lock_script_hash2, asset_type, 15),
            ]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(Ok(()), state.apply(&transfer, &registrar, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, registrar: registrar }),
            (asset: (mint_tracker, 0)),
            (asset: (transfer_tracker, 0) => { asset_type: asset_type, quantity: 10 }),
            (asset: (transfer_tracker, 1) => { asset_type: asset_type, quantity: 5 }),
            (asset: (transfer_tracker, 2) => { asset_type: asset_type, quantity: 15 })
        ]);
    }

    #[test]
    fn registrar_can_burn() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let registrar = address();
        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let amount = 30;
        let mint =
            asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata.clone(), registrar: registrar);
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, registrar: registrar }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let burn =
            asset_transfer!(burns: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, amount))]);
        let burn_tracker = burn.tracker();

        assert_eq!(Ok(()), state.apply(&burn, &registrar, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: 0, registrar: registrar }),
            (asset: (mint_tracker, 0)),
            (asset: (burn_tracker, 0))
        ]);
    }

    #[test]
    fn cannot_transfer_because_prev_out_amount_is_invalid() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let amount = 30;
        let mint = asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata.clone());
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 20), vec![0x30, 0x01])],
            asset_transfer_outputs![(lock_script_hash, vec![vec![1]], asset_type, 20)]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(
            Err(StateError::Runtime(RuntimeError::InvalidAssetQuantity {
                shard_id: SHARD_ID,
                tracker: mint_tracker,
                index: 0,
                expected: 30,
                got: 20
            })),
            state.apply(&transfer, &sender, &[sender], &[], &get_test_client(), 0, 0)
        );

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount }),
            (asset: (transfer_tracker, 0))
        ]);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn cannot_transfer_because_prev_out_type_is_invalid() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let amount = 30;

        let metadata1 = "metadata".to_string();
        let mint1 = asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata1.clone());
        let mint_tracker1 = mint1.tracker();
        let asset_type1 = Blake::blake(*mint_tracker1);

        let metadata2 = "metadata2".to_string();
        let mint2 = asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata2.clone());
        let mint_tracker2 = mint2.tracker();
        let asset_type2 = Blake::blake(*mint_tracker2);

        assert_eq!(Ok(()), state.apply(&mint1, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type1) => { metadata: metadata1, supply: amount }),
            (scheme: (asset_type2)),
            (asset: (mint_tracker1, 0) => { asset_type: asset_type1, quantity: amount })
        ]);

        assert_eq!(Ok(()), state.apply(&mint2, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type1) => { metadata: metadata1, supply: amount }),
            (scheme: (asset_type2) => { metadata: metadata2, supply: amount }),
            (asset: (mint_tracker1, 0) => { asset_type: asset_type1, quantity: amount }),
            (asset: (mint_tracker2, 0) => { asset_type: asset_type2, quantity: amount })
        ]);

        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker1, 0, asset_type2, 30), vec![0x30, 0x01])],
            asset_transfer_outputs![(lock_script_hash, vec![vec![1]], asset_type2, 30)]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(
            Err(StateError::Runtime(RuntimeError::UnexpectedAssetType {
                index: 0,
                mismatch: Mismatch {
                    found: asset_type2,
                    expected: asset_type1,
                }
            })),
            state.apply(&transfer, &sender, &[sender], &[], &get_test_client(), 0, 0)
        );

        check_shard_level_state!(state, [
            (scheme: (asset_type1) => { metadata: metadata1, supply: amount }),
            (scheme: (asset_type2) => { metadata: metadata2, supply: amount }),
            (asset: (mint_tracker1, 0) => { asset_type: asset_type1, quantity: amount }),
            (asset: (mint_tracker2, 0) => { asset_type: asset_type2, quantity: amount }),
            (asset: (transfer_tracker, 0))
        ]);
    }

    #[test]
    fn wrap_and_unwrap_ccc() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let lock_script_hash = H160::from("ca5d3fa0a6887285ef6aa85cb12960a2b6706e00");
        let tx_hash = TxHash::from(H256::random());
        let amount = 30;

        let wrap_ccc = asset_wrap_ccc!(tx_hash, asset_wrap_ccc_output!(lock_script_hash, amount));
        let wrap_ccc_tracker = wrap_ccc.tracker();
        let asset_type = H160::zero();

        assert_eq!(*wrap_ccc_tracker, *tx_hash);
        assert_eq!(Ok(()), state.apply(&wrap_ccc, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { supply: amount }),
            (asset: (wrap_ccc_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let unwrap_amount = 30;
        let unwrap_ccc = asset_unwrap_ccc!(
            asset_transfer_input!(asset_out_point!(wrap_ccc_tracker, 0, asset_type, unwrap_amount), vec![0x01]),
            sender
        );

        assert_eq!(Ok(()), state.apply(&unwrap_ccc, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { supply: amount - unwrap_amount }),
            (asset: (wrap_ccc_tracker, 0))
        ]);
    }

    #[test]
    fn wrap_ccc_and_transfer_and_unwrap_ccc() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let tx_hash = TxHash::from(H256::random());
        let amount = 30;

        let wrap_ccc = asset_wrap_ccc!(tx_hash, asset_wrap_ccc_output!(lock_script_hash, amount));
        let wrap_ccc_tracker = wrap_ccc.tracker();

        assert_eq!(*wrap_ccc_tracker, *tx_hash);
        assert_eq!(Ok(()), state.apply(&wrap_ccc, &sender, &[sender], &[], &get_test_client(), 0, 0));

        let asset_type = H160::zero();

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { supply: amount }),
            (asset: (wrap_ccc_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let lock_script_hash_burn = H160::from("ca5d3fa0a6887285ef6aa85cb12960a2b6706e00");
        let random_lock_script_hash = H160::random();
        let transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(wrap_ccc_tracker, 0, asset_type, 30), vec![0x30, 0x01])],
            asset_transfer_outputs![
                (lock_script_hash, vec![vec![1]], asset_type, 10),
                (lock_script_hash_burn, asset_type, 5),
                (random_lock_script_hash, asset_type, 15),
            ]
        );
        let transfer_tracker = transfer.tracker();

        assert_eq!(Ok(()), state.apply(&transfer, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { supply: amount }),
            (asset: (wrap_ccc_tracker, 0)),
            (asset: (transfer_tracker, 0) => { asset_type: asset_type, quantity: 10 }),
            (asset: (transfer_tracker, 1) => { asset_type: asset_type, quantity: 5 }),
            (asset: (transfer_tracker, 2) => { asset_type: asset_type, quantity: 15 })
        ]);

        let unwrap_amount = 5;
        let unwrap_ccc = asset_unwrap_ccc!(
            asset_transfer_input!(asset_out_point!(transfer_tracker, 1, asset_type, unwrap_amount), vec![0x01]),
            sender
        );

        assert_eq!(Ok(()), state.apply(&unwrap_ccc, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { supply: amount - unwrap_amount }),
            (asset: (wrap_ccc_tracker, 0)),
            (asset: (transfer_tracker, 0) => { asset_type: asset_type, quantity: 10 }),
            (asset: (transfer_tracker, 1)),
            (asset: (transfer_tracker, 2) => { asset_type: asset_type, quantity: 15 })
        ]);
    }

    #[test]
    fn mint_and_failed_transfer_and_successful_transfer() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::from("b042ad154a3359d276835c903587ebafefea22af");
        let amount = 30;
        let mint = asset_mint!(asset_mint_output!(lock_script_hash, supply: amount), metadata.clone());
        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata.clone(), supply: amount }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let failed_lock_script = vec![0x30];
        let failed_transfer = asset_transfer!(
            inputs:
                asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 30), failed_lock_script.clone())],
            asset_transfer_outputs![(lock_script_hash, vec![vec![1]], asset_type, 30)]
        );
        let failed_transfer_tracker = failed_transfer.tracker();

        let sender = address();
        assert_eq!(
            Err(StateError::Runtime(RuntimeError::ScriptHashMismatch(Mismatch {
                expected: lock_script_hash,
                found: Blake::blake(&failed_lock_script),
            }))),
            state.apply(&failed_transfer, &sender, &[sender], &[], &get_test_client(), 0, 0)
        );

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata.clone(), supply: amount }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount }),
            (asset: (failed_transfer_tracker, 0))
        ]);

        let random_lock_script_hash = H160::random();
        let successful_transfer = asset_transfer!(
            inputs: asset_transfer_inputs![(asset_out_point!(mint_tracker, 0, asset_type, 30), vec![0x30, 0x01])],
            asset_transfer_outputs![
                (lock_script_hash, vec![vec![1]], asset_type, 10),
                (lock_script_hash, asset_type, 5),
                (random_lock_script_hash, asset_type, 15),
            ]
        );
        let successful_transfer_tracker = successful_transfer.tracker();

        assert_eq!(Ok(()), state.apply(&successful_transfer, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount }),
            (asset: (mint_tracker, 0)),
            (asset: (failed_transfer_tracker, 0)),
            (asset: (successful_transfer_tracker, 0) => { asset_type: asset_type, quantity: 10 }),
            (asset: (successful_transfer_tracker, 1) => { asset_type: asset_type, quantity: 5 }),
            (asset: (successful_transfer_tracker, 2) => { asset_type: asset_type, quantity: 15 })
        ]);
    }

    #[test]
    fn users_can_mint_asset() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let approver = Address::random();
        let transaction = asset_mint!(
            asset_mint_output!(lock_script_hash, parameters: parameters),
            metadata.clone(),
            approver: approver
        );
        let transaction_tracker = transaction.tracker();
        let asset_type = Blake::blake(*transaction_tracker);

        assert_eq!(Ok(()), state.apply(&transaction, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: ::std::u64::MAX }),
            (asset: (transaction_tracker, 0) => { asset_type: asset_type, quantity: ::std::u64::MAX })
        ]);
    }

    #[test]
    fn mint_is_failed_when_shard_users_are_disjoint_to_sender_and_approvers() {
        let shard_users = [address(), address(), address()];
        let sender = address();
        let approvers = [address(), address(), address()];

        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let approver = Address::random();
        let transaction =
            asset_mint!(asset_mint_output!(lock_script_hash, parameters: parameters), metadata, approver: approver);

        let transaction_tracker = transaction.tracker();
        let asset_type = Blake::blake(*transaction_tracker);

        assert_eq!(
            Err(StateError::Runtime(RuntimeError::InsufficientPermission)),
            state.apply(&transaction, &sender, &shard_users, &approvers, &get_test_client(), 0, 0)
        );

        check_shard_level_state!(state, [
            (scheme: (asset_type)),
            (asset: (transaction_tracker, 0))
        ]);
    }

    #[test]
    fn can_mint_when_shard_user_sent_a_transaction() {
        let shard_users = [address(), address(), address()];
        let sender = shard_users[0];
        let approvers = [address(), address(), address()];

        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let approver = Address::random();
        let transaction = asset_mint!(
            asset_mint_output!(lock_script_hash, parameters: parameters),
            metadata.clone(),
            approver: approver
        );

        let transaction_tracker = transaction.tracker();
        let asset_type = Blake::blake(*transaction_tracker);

        check_shard_level_state!(state, [
            (scheme: (asset_type)),
            (asset: (transaction_tracker, 0))
        ]);

        assert_eq!(Ok(()), state.apply(&transaction, &sender, &shard_users, &approvers, &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: ::std::u64::MAX }),
            (asset: (transaction_tracker, 0) => { asset_type: asset_type, quantity: ::std::u64::MAX })
        ]);
    }

    #[test]
    fn can_mint_when_shard_user_approves_a_transaction() {
        let shard_users = [address(), address(), address()];
        let sender = address();
        let approvers = [shard_users[0], address(), address()];

        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let approver = Address::random();
        let transaction = asset_mint!(
            asset_mint_output!(lock_script_hash, parameters: parameters),
            metadata.clone(),
            approver: approver
        );

        let transaction_tracker = transaction.tracker();
        let asset_type = Blake::blake(*transaction_tracker);

        check_shard_level_state!(state, [
            (scheme: (asset_type)),
            (asset: (transaction_tracker, 0))
        ]);

        assert_eq!(Ok(()), state.apply(&transaction, &sender, &shard_users, &approvers, &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: ::std::u64::MAX }),
            (asset: (transaction_tracker, 0) => { asset_type: asset_type, quantity: ::std::u64::MAX })
        ]);
    }

    #[test]
    fn anyone_can_mint_if_no_users() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let approver = Address::random();
        let transaction = asset_mint!(
            asset_mint_output!(lock_script_hash, parameters: parameters),
            metadata.clone(),
            approver: approver
        );

        let transaction_tracker = transaction.tracker();
        let asset_type = Blake::blake(*transaction_tracker);

        assert_eq!(Ok(()), state.apply(&transaction, &sender, &[], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: ::std::u64::MAX, approver: approver }),
            (asset: (transaction_tracker, 0) => { asset_type: asset_type, quantity: ::std::u64::MAX })
        ]);
    }

    #[test]
    fn change_asset_scheme() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let amount = 100;
        let registrar = Address::random();
        let mint = asset_mint!(
            asset_mint_output!(lock_script_hash, parameters, amount),
            metadata.clone(),
            registrar: registrar
        );

        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, approver, registrar: registrar }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let approver = Address::random();
        let change_asset_scheme = ShardTransaction::ChangeAssetScheme {
            network_id: "tc".into(),
            shard_id: SHARD_ID,
            asset_type,
            seq: 0,
            metadata: "New metadata".to_string(),
            approver: Some(approver),
            registrar: None,
            allowed_script_hashes: Vec::new(),
        };
        assert_eq!(Ok(()), state.apply(&change_asset_scheme, &sender, &[], &[registrar], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: "New metadata".to_string(), supply: amount, approver: approver, registrar }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);
    }

    #[test]
    fn increase_asset_amount() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let metadata = "metadata".to_string();
        let lock_script_hash = H160::random();
        let parameters = vec![];
        let amount = 100;
        let registrar = Address::random();
        let mint = asset_mint!(
            asset_mint_output!(lock_script_hash, parameters, amount),
            metadata.clone(),
            registrar: registrar
        );

        let mint_tracker = mint.tracker();
        let asset_type = Blake::blake(*mint_tracker);

        assert_eq!(Ok(()), state.apply(&mint, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: metadata, supply: amount, approver, registrar: registrar }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount })
        ]);

        let new_supply = 200;
        let increase_supply = ShardTransaction::IncreaseAssetSupply {
            network_id: "tc".into(),
            shard_id: SHARD_ID,
            asset_type,
            seq: 0,
            output: AssetMintOutput {
                lock_script_hash: H160::random(),
                parameters: vec![],
                supply: new_supply,
            },
        };
        let supply_tracker = increase_supply.tracker();

        assert_eq!(Ok(()), state.apply(&increase_supply, &sender, &[], &[registrar], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (scheme: (asset_type) => { metadata: "metadata".to_string(), supply: amount + new_supply, approver, registrar: registrar }),
            (asset: (mint_tracker, 0) => { asset_type: asset_type, quantity: amount }),
            (asset: (supply_tracker, 0) => { asset_type: asset_type, quantity: new_supply })
        ]);
    }

    #[test]
    fn store_shard_text() {
        let sender = address();
        let mut state_db = RefCell::new(get_temp_state_db());
        let mut shard_cache = ShardCache::default();
        let mut state = get_temp_shard_state(&mut state_db, SHARD_ID, &mut shard_cache);

        let content = "stored text".to_string();

        let store_shard_text = ShardTransaction::ShardStore {
            network_id: "tc".into(),
            shard_id: crate::impls::test_helper::SHARD_ID,
            content: content.clone(),
        };

        let store_shard_text_tracker = store_shard_text.tracker();

        assert_eq!(Ok(()), state.apply(&store_shard_text, &sender, &[sender], &[], &get_test_client(), 0, 0));

        check_shard_level_state!(state, [
            (text: (store_shard_text_tracker) => { content: &content })
        ]);
    }
}
