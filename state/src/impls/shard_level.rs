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
use crate::{Asset, AssetScheme, AssetSchemeAddress, ShardText, ShardTextAddress, StateDB, StateResult};
use ccrypto::BLAKE_NULL_RLP;
use cdb::AsHashDB;
use ckey::Address;
use ctypes::transaction::ShardTransaction;
use ctypes::{BlockNumber, ShardId, Tracker};
use cvm::ChainTimeInfo;
use merkle_trie::{Result as TrieResult, TrieError, TrieFactory};
use primitives::{H160, H256};
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
    fn text(&self, tracker: Tracker) -> Result<Option<ShardText>, TrieError> {
        let db = self.db.borrow();
        let trie = TrieFactory::readonly(db.as_hashdb(), &self.root)?;
        self.cache.shard_text(&ShardTextAddress::new(tracker, self.shard_id), &trie)
    }
}

#[cfg(test)]
mod tests {
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
