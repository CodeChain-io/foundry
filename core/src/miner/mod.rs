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

use ckey::{Ed25519Public as Public, Password, PlatformAddress};
use cstate::TopStateView;
use ctypes::transaction::IncompleteTransaction;
use ctypes::{BlockHash, BlockId, TxHash};
use primitives::Bytes;
use std::ops::Range;

use self::mem_pool_types::AccountDetails;
pub use self::miner::{AuthoringParams, Miner, MinerOptions};
use crate::account_provider::{AccountProvider, Error as AccountProviderError};
use crate::client::{
    AccountData, BlockChainTrait, BlockProducer, EngineInfo, ImportBlock, MiningBlockChainClient, TermInfo,
};
use crate::consensus::EngineType;
use crate::error::Error;
use crate::transaction::{PendingVerifiedTransactions, UnverifiedTransaction, VerifiedTransaction};

/// Miner client API
pub trait MinerService: Send + Sync {
    /// Type representing chain state
    type State: TopStateView + 'static;

    /// Returns the number of pending transactions.
    fn num_pending_transactions(&self) -> usize;

    /// Get current authoring parameters.
    fn authoring_params(&self) -> AuthoringParams;

    /// Set the author that we will seal blocks as.
    fn set_author(&self, author: Public) -> Result<(), AccountProviderError>;

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
        C: AccountData + BlockChainTrait + BlockProducer + EngineInfo + ImportBlock;

    /// Get the type of consensus engine.
    fn engine_type(&self) -> EngineType;

    /// New chain head event. Restart mining operation.
    fn update_sealing<C>(&self, chain: &C, parent_block: BlockId, allow_empty_block: bool)
    where
        C: AccountData + BlockChainTrait + BlockProducer + ImportBlock + EngineInfo + TermInfo;

    /// Imports transactions to mem pool.
    fn import_external_transactions<C: MiningBlockChainClient + EngineInfo + TermInfo>(
        &self,
        client: &C,
        transactions: Vec<UnverifiedTransaction>,
    ) -> Vec<Result<TransactionImportResult, Error>>;

    /// Imports own (node owner) transaction to mem pool.
    fn import_own_transaction<C: MiningBlockChainClient + EngineInfo + TermInfo>(
        &self,
        chain: &C,
        tx: VerifiedTransaction,
    ) -> Result<TransactionImportResult, Error>;

    /// Imports incomplete (node owner) transaction to mem pool.
    fn import_incomplete_transaction<C: MiningBlockChainClient + AccountData + EngineInfo + TermInfo>(
        &self,
        chain: &C,
        account_provider: &AccountProvider,
        tx: IncompleteTransaction,
        platform_address: PlatformAddress,
        passphrase: Option<Password>,
        seq: Option<u64>,
    ) -> Result<(TxHash, u64), Error>;

    /// Get a list of all pending transactions in the mem pool.
    fn ready_transactions(&self, size_limit: usize, range: Range<u64>) -> PendingVerifiedTransactions;

    /// Get a count of all pending transactions in the mem pool.
    fn count_pending_transactions(&self, range: Range<u64>) -> usize;

    /// Start sealing.
    fn start_sealing<C: MiningBlockChainClient + EngineInfo + TermInfo>(&self, client: &C);

    /// Stop sealing.
    fn stop_sealing(&self);
}

/// Represents the result of importing tranasction.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum TransactionImportResult {
    /// Tranasction was imported to current queue.
    Current,
    /// Transaction was imported to future queue.
    Future,
}

fn fetch_account_creator<'c>(
    client: &'c dyn AccountData,
    block_id: BlockId,
) -> impl Fn(&Public) -> AccountDetails + 'c {
    move |pubkey: &Public| AccountDetails {
        seq: client.seq(&pubkey, block_id).expect("We are querying sequence using trusted block id"),
        balance: client.balance(&pubkey, block_id.into()).expect("We are querying balance using trusted block id"),
    }
}
