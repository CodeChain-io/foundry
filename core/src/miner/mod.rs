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

mod backup;
mod mem_pool;
mod mem_pool_types;
#[cfg_attr(feature = "cargo-clippy", allow(clippy::module_inception))]
mod miner;

use ckey::Address;
use cstate::TopStateView;
use ctypes::BlockHash;
use primitives::Bytes;
use std::ops::Range;
use std::sync::Arc;

pub use self::miner::{AuthoringParams, Miner, MinerOptions};
use crate::account_provider::{AccountProvider, Error as AccountProviderError};
use crate::client::{BlockChainClient, BlockChainTrait, BlockProducer, EngineInfo, ImportBlock, TermInfo};
use crate::consensus::EngineType;
use crate::error::Error;
use crate::transaction::PendingTransactions;
use crate::BlockId;
use coordinator::validator::Transaction;

/// Miner client API
pub trait MinerService: Send + Sync {
    /// Type representing chain state
    type State: TopStateView + 'static;

    /// Returns miner's status.
    fn status(&self) -> MinerStatus;

    /// Get current authoring parameters.
    fn authoring_params(&self) -> AuthoringParams;

    /// Set the author that we will seal blocks as.
    fn set_author(&self, ap: Arc<AccountProvider>, author: Address) -> Result<(), AccountProviderError>;

    ///Get the address that sealed the block.
    fn get_author_address(&self) -> Address;

    /// Set the extra_data that we will seal blocks with.
    fn set_extra_data(&self, extra_data: Bytes);

    /// Get current transactions limit in queue.
    fn transactions_limit(&self) -> usize;

    /// Set maximal number of transactions kept in the queue (both current and future).
    fn set_transactions_limit(&self, limit: usize);

    /// Called when blocks are imported to chain, updates transactions queue.
    fn chain_new_blocks<C>(&self, chain: &C, imported: &[BlockHash], invalid: &[BlockHash], enacted: &[BlockHash])
    where
        C: BlockChainTrait + BlockProducer + EngineInfo + ImportBlock;

    /// Get the type of consensus engine.
    fn engine_type(&self) -> EngineType;

    /// New chain head event. Restart mining operation.
    fn update_sealing<C>(&self, chain: &C, parent_block: BlockId, allow_empty_block: bool)
    where
        C: BlockChainTrait + BlockProducer + ImportBlock + EngineInfo + TermInfo;

    /// Imports transactions to mem pool.
    fn import_external_transactions<C: BlockChainClient + BlockProducer + EngineInfo + TermInfo>(
        &self,
        client: &C,
        transactions: Vec<Transaction>,
    ) -> Vec<Result<(), Error>>;

    /// Imports own (node owner) transaction to mem pool.
    fn import_own_transaction<C: BlockChainClient + BlockProducer + EngineInfo + TermInfo>(
        &self,
        chain: &C,
        tx: Transaction,
    ) -> Result<(), Error>;

    /// Get a list of all pending transactions in the mem pool.
    fn ready_transactions(&self, gas_limit: usize, size_limit: usize, range: Range<u64>) -> PendingTransactions;

    /// Get a count of all pending transactions in the mem pool.
    fn count_pending_transactions(&self, range: Range<u64>) -> usize;

    /// Start sealing.
    fn start_sealing<C: BlockChainClient + BlockProducer + EngineInfo + TermInfo>(&self, client: &C);

    /// Stop sealing.
    fn stop_sealing(&self);
}

/// Mining status
#[derive(Debug)]
pub struct MinerStatus {
    /// Number of transactions in queue with state `pending` (ready to be included in block)
    pub transactions_in_pending_queue: usize,
}

#[cfg(all(feature = "nightly", test))]
mod mem_pool_benches;
