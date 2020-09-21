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

mod backup;
mod mem_pool;
mod mem_pool_types;
#[cfg_attr(feature = "cargo-clippy", allow(clippy::module_inception))]
mod miner;

use ckey::Ed25519Public as Public;
use cstate::TopStateView;
use ctypes::{BlockHash, BlockId};
use primitives::Bytes;
use std::ops::Range;
use std::sync::Arc;

pub use self::miner::{AuthoringParams, Miner, MinerOptions};
use crate::account_provider::{AccountProvider, Error as AccountProviderError};
use crate::client::{BlockChainTrait, BlockProducer, EngineInfo, ImportBlock, MiningBlockChainClient, TermInfo};
use crate::consensus::EngineType;
use crate::error::Error;
use crate::{PendingTransactions, StateInfo};
use coordinator::Transaction;

/// Miner client API
pub trait MinerService: Send + Sync {
    /// Type representing chain state
    type State: TopStateView + 'static;

    /// Returns the number of pending transactions.
    fn num_pending_transactions(&self) -> usize;

    /// Get current authoring parameters.
    fn authoring_params(&self) -> AuthoringParams;

    /// Set the author that we will seal blocks as.
    fn set_author(&self, ap: Arc<AccountProvider>, author: Public) -> Result<(), AccountProviderError>;

    ///Get the address of block author.
    fn get_author(&self) -> Public;

    /// Set the extra_data that we will seal blocks with.
    fn set_extra_data(&self, extra_data: Bytes);

    /// Get current transactions limit in queue.
    fn transactions_limit(&self) -> usize;

    /// Set maximal number of transactions kept in the queue (both current and future).
    fn set_transactions_limit(&self, limit: usize);

    /// Called when blocks are imported to chain, updates transactions queue.
    fn chain_new_blocks<C>(&self, chain: &C, imported: &[BlockHash], invalid: &[BlockHash], enacted: &[BlockHash])
    where
        C: BlockChainTrait + BlockProducer + EngineInfo + ImportBlock + StateInfo;

    /// Get the type of consensus engine.
    fn engine_type(&self) -> EngineType;

    /// New chain head event. Restart mining operation.
    fn update_sealing<C>(&self, chain: &C, parent_block: BlockId, allow_empty_block: bool)
    where
        C: BlockChainTrait + BlockProducer + ImportBlock + EngineInfo + TermInfo;

    /// Imports transactions to mem pool.
    fn import_external_transactions<C: MiningBlockChainClient + EngineInfo + TermInfo + StateInfo>(
        &self,
        client: &C,
        transactions: Vec<Transaction>,
    ) -> Vec<Result<(), Error>>;

    /// Imports own (node owner) transaction to mem pool.
    fn import_own_transaction<C: MiningBlockChainClient + EngineInfo + TermInfo + StateInfo>(
        &self,
        chain: &C,
        tx: Transaction,
    ) -> Result<(), Error>;

    /// Get a list of all pending transactions in the mem pool.
    fn pending_transactions(&self, size_limit: usize, range: Range<u64>) -> PendingTransactions;

    /// Get a count of all pending transactions.
    fn count_pending_transactions(&self, range: Range<u64>) -> usize;

    /// Start sealing.
    fn start_sealing<C: MiningBlockChainClient + EngineInfo + TermInfo>(&self, client: &C);

    /// Stop sealing.
    fn stop_sealing(&self);
}
