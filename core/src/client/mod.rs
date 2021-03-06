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

mod chain_notify;
#[cfg_attr(feature = "cargo-clippy", allow(clippy::module_inception))]
mod client;
mod config;
mod importer;
pub mod snapshot_notify;
mod test_client;

pub use self::chain_notify::ChainNotify;

pub use self::client::Client;
pub use self::config::ClientConfig;
pub use self::test_client::TestBlockChainClient;

use crate::block::{Block, ClosedBlock, OpenBlock};
use crate::blockchain_info::BlockChainInfo;
use crate::consensus::EngineError;
use crate::encoded;
use crate::error::{BlockImportError, Error as GenericError};
use crate::transaction::{LocalizedTransaction, PendingTransactions};
use crate::types::{BlockStatus, TransactionId, VerificationQueueInfo as BlockQueueInfo};
use cdb::DatabaseError;
use ckey::{Ed25519Public as Public, NetworkId, PlatformAddress};
use coordinator::types::Event;
use coordinator::Transaction;
use cstate::{TopLevelState, TopStateView};
use ctypes::{BlockHash, BlockId, BlockNumber, ChainParams, CompactValidatorSet, Header, SyncHeader, TxHash};
use kvdb::KeyValueDB;
use primitives::Bytes;
use std::ops::Range;
use std::sync::Arc;

/// Provides various blockchain information, like block header, chain state etc.
pub trait BlockChainTrait {
    /// Get blockchain information.
    fn chain_info(&self) -> BlockChainInfo;

    /// Get raw block header data by block id.
    fn block_header(&self, id: &BlockId) -> Option<encoded::Header>;

    /// Get the best block header.
    fn best_block_header(&self) -> encoded::Header;

    /// Get the best header. Note that this is different from best block's header.
    fn best_header(&self) -> encoded::Header;

    /// Get the best proposal block header.
    fn best_proposal_header(&self) -> encoded::Header;

    /// Get raw block data by block header hash.
    fn block(&self, id: &BlockId) -> Option<encoded::Block>;

    /// Get the hash of block that contains the transaction, if any.
    fn transaction_block(&self, id: &TransactionId) -> Option<BlockHash>;
}

pub trait EngineInfo: Send + Sync {
    fn network_id(&self) -> NetworkId;
    fn chain_params(&self, block_id: BlockId) -> Option<ChainParams>;
    fn possible_authors(&self, block_number: Option<u64>) -> Result<Option<Vec<PlatformAddress>>, EngineError>;
    fn validator_set(&self, block_number: Option<u64>) -> Result<Option<CompactValidatorSet>, EngineError>;
}

/// Client facilities used by internally sealing Engines.
pub trait EngineClient: Sync + Send + BlockChainTrait + ImportBlock {
    /// Make a new block and seal it.
    fn update_sealing(&self, parent_block: BlockId, allow_empty_block: bool);

    /// Update the best block as the given block hash
    ///
    /// Used in Tendermint, when going to the commit step.
    fn update_best_as_committed(&self, block_hash: BlockHash);

    fn get_kvdb(&self) -> Arc<dyn KeyValueDB>;
}

pub trait ConsensusClient: BlockChainClient + EngineClient + EngineInfo + StateInfo {}

/// State information to be used during client query
pub enum StateOrBlock {
    /// State to be used, may be pending
    State(Box<dyn TopStateView>),

    /// Id of an existing block from a chain to get state from
    Block(BlockId),
}

impl From<Box<dyn TopStateView>> for StateOrBlock {
    fn from(info: Box<dyn TopStateView>) -> StateOrBlock {
        StateOrBlock::State(info)
    }
}

impl From<BlockId> for StateOrBlock {
    fn from(id: BlockId) -> StateOrBlock {
        StateOrBlock::Block(id)
    }
}

/// Provides methods to import block into blockchain
pub trait ImportBlock {
    /// Import a block into the blockchain.
    fn import_block(&self, bytes: Bytes) -> Result<BlockHash, BlockImportError>;

    /// Import a header into the blockchain
    fn import_header(&self, header: SyncHeader) -> Result<BlockHash, BlockImportError>;

    /// Import a trusted header into the blockchain
    /// Trusted header doesn't go through any verifications and doesn't update the best header
    /// The trusted header may not have parent.
    fn import_trusted_header(&self, header: Header) -> Result<BlockHash, BlockImportError>;

    /// Import a trusted block into the blockchain
    /// Trusted block doesn't go through any verifications and doesn't update the best block
    /// The trusted block may not have parent.
    fn import_trusted_block(&self, block: &Block) -> Result<BlockHash, BlockImportError>;

    /// Forcefully update the best block
    fn force_update_best_block(&self, hash: &BlockHash);

    /// Import closed block. Skips all verifications. This block is generated by this instance.
    fn import_generated_block(&self, block: &ClosedBlock) -> ImportResult;

    /// Set reseal min timer as reseal_min_period, for creating blocks with transactions which are pending because of reseal_min_period
    fn set_min_timer(&self);
}

/// Blockchain database client. Owns and manages a blockchain and a block queue.
pub trait BlockChainClient: Sync + Send + BlockChainTrait + ImportBlock {
    /// Get block queue information.
    fn queue_info(&self) -> BlockQueueInfo;

    /// Queue own transaction to mem_pool for importing
    fn queue_own_transaction(&self, transaction: Transaction) -> Result<(), GenericError>;

    /// Queue transactions to mem_pool for importing.
    fn queue_transactions(&self, transactions: Vec<Bytes>);

    /// Delete all pending transactions.
    fn delete_all_pending_transactions(&self);

    /// List all transactions in the mem_pool a.k.a pending transactions
    fn pending_transactions(&self, range: Range<u64>) -> PendingTransactions;

    /// Get the count of all pending transactions currently in the mem_pool.
    fn count_pending_transactions(&self, range: Range<u64>) -> usize;

    /// Check whether there is any pending transactions or not.
    fn is_mem_pool_empty(&self) -> bool;

    /// Look up the block number for the given block ID.
    fn block_number(&self, id: &BlockId) -> Option<BlockNumber>;

    /// Get raw block body data by block id.
    /// Block body is an RLP list of one item: transactions.
    fn block_body(&self, id: &BlockId) -> Option<encoded::Body>;

    /// Get block status by block header hash.
    fn block_status(&self, id: &BlockId) -> BlockStatus;

    /// Get block hash.
    fn block_hash(&self, id: &BlockId) -> Option<BlockHash>;

    /// Get transaction with given hash.
    fn transaction(&self, id: &TransactionId) -> Option<LocalizedTransaction>;

    /// get events emitted by given transaction
    fn events_by_tx_hash(&self, hash: &TxHash) -> Vec<Event>;

    /// get events emitted by given block
    fn events_by_block_hash(&self, hash: &BlockHash) -> Vec<Event>;
}

/// Result of import block operation.
pub type ImportResult = Result<BlockHash, DatabaseError>;

/// Provides methods used for sealing new state
pub trait BlockProducer {
    /// Returns OpenBlock prepared for closing.
    fn prepare_open_block(&self, parent_block: BlockId, author: Public, extra_data: Bytes) -> OpenBlock;
}

/// Extended client interface used for mining
pub trait MiningBlockChainClient: BlockChainClient + BlockProducer {}

/// Provides methods to access database.
pub trait DatabaseClient {
    fn database(&self) -> Arc<dyn KeyValueDB>;
}

pub trait StateInfo {
    /// Attempt to get a copy of a specific block's final state.
    ///
    /// This will not fail if given BlockId::Latest.
    /// Otherwise, this can fail (but may not) if the DB prunes state or the block
    /// is unknown.
    fn state_at(&self, id: BlockId) -> Option<TopLevelState>;
}

pub trait SnapshotClient {
    fn notify_snapshot(&self, id: BlockId);
}
