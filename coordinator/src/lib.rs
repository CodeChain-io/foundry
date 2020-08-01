// Copyright 2020 Kodebox, Inc.
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

mod app_desc;
pub mod context;
pub mod engine;
mod header;
pub mod module;
mod substorage;
pub mod test_coordinator;
mod transaction;
pub mod types;
mod values;

use self::context::{StorageAccess, SubStorageAccess};
use self::engine::{BlockExecutor, Initializer, TxFilter};
pub use self::header::Header;
pub use self::transaction::{Transaction, TransactionWithMetadata, TxOrigin};
use self::types::{BlockOutcome, ErrorCode, VerifiedCrime};
use crate::engine::FilteredTxs;
use crate::module::{HandleCrimes, InitChain, InitGenesis, SortedTxs, Stateful, TxOwner, TxSorter, UpdateChain};
use crate::substorage::SubStorageView;
use crate::types::{CloseBlockError, ExecuteTransactionError, HeaderError, TransactionOutcome};
use ctypes::StorageId;
use ctypes::{CompactValidatorSet, ConsensusParams};
use parking_lot::Mutex;
use remote_trait_object::ServiceRef;
use std::collections::HashMap;
use std::sync::Arc;

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.
pub struct Coordinator {
    /// The maximum block size.
    max_body_size: usize,

    /// The key services from modules for implementing a chain.
    inner: Mutex<Inner>,
}

struct Inner {
    /// The current storage set to all `Stateful` modules.
    current_storage: Option<Arc<Mutex<dyn StorageAccess>>>,

    /// List of module name and `Stateful` service pairs in the current app.
    /// The module name is used to keep the index of the corresponding `Stateful`
    /// same across updates, since the index is used as `StorageId`.
    stateful: Vec<(String, Box<dyn Stateful>)>,

    /// List of module name and its `InitGenesis` pairs.
    init_genesis: Vec<(String, Box<dyn InitGenesis>)>,

    /// Per-module genesis config.
    genesis_config: HashMap<String, Vec<u8>>,

    /// A map from Tx type to its owner.
    tx_owner: HashMap<String, Box<dyn TxOwner>>,

    /// An optional crime handler.
    handle_crimes: Option<Box<dyn HandleCrimes>>,

    /// List of module name and its `InitChain` pairs.
    init_chain: Box<dyn InitChain>,

    /// List of module name and its `UpdateChain` pairs.
    update_chain: Box<dyn UpdateChain>,

    /// A service sorting Tx'es in the mempool.
    tx_sorter: Box<dyn TxSorter>,
}

impl Inner {
    fn set_storage(&mut self, storage: &Arc<Mutex<dyn StorageAccess>>) {
        self.current_storage = Some(Arc::clone(storage));
        for (id, (_name, ref mut stateful)) in self.stateful.iter_mut().enumerate() {
            let substorage: Box<dyn SubStorageAccess> =
                Box::new(SubStorageView::of(id as StorageId, Arc::clone(storage)));
            stateful.set_storage(ServiceRef::create_export(substorage));
        }
    }
}

impl Initializer for Coordinator {
    fn initialize_chain(&self, storage: Arc<Mutex<dyn StorageAccess>>) -> (CompactValidatorSet, ConsensusParams) {
        let inner = &mut *self.inner.lock();

        inner.set_storage(&storage);

        for (_, init) in inner.init_genesis.iter_mut() {
            init.begin_genesis();
        }

        for (ref module, ref mut init) in inner.init_genesis.iter_mut() {
            let config = match inner.genesis_config.get(module) {
                Some(value) => value as &[u8],
                None => &[],
            };
            init.init_genesis(config);
        }

        for (_, init) in inner.init_genesis.iter_mut() {
            init.end_genesis();
        }

        inner.init_chain.init_chain()
    }
}

impl BlockExecutor for Coordinator {
    fn open_block(
        &self,
        storage: Arc<Mutex<dyn StorageAccess>>,
        header: &Header,
        verified_crimes: &[VerifiedCrime],
    ) -> Result<(), HeaderError> {
        let mut inner = self.inner.lock();

        inner.set_storage(&storage);

        if let Some(ref mut handle_crimes) = inner.handle_crimes {
            handle_crimes.handle_crimes(verified_crimes);
        }

        for owner in inner.tx_owner.values_mut() {
            owner.block_opened(header)?;
        }

        Ok(())
    }

    fn execute_transactions(
        &self,
        transactions: &[Transaction],
    ) -> Result<Vec<TransactionOutcome>, ExecuteTransactionError> {
        let inner = &mut *self.inner.lock();

        let storage = match inner.current_storage {
            Some(ref storage) => storage,
            None => panic!("A StorageAccess should've been set via a call to open_block"),
        };
        let storage = &mut *storage.lock();

        let mut outcomes = Vec::with_capacity(transactions.len());

        for tx in transactions {
            match inner.tx_owner.get_mut(tx.tx_type()) {
                Some(owner) => {
                    storage.create_checkpoint();
                    match owner.execute_transaction(tx) {
                        Ok(outcome) => {
                            outcomes.push(outcome);
                            storage.discard_checkpoint();
                        }
                        Err(_) => storage.revert_to_the_checkpoint(),
                    }
                }
                None => outcomes.push(TransactionOutcome::default()),
            }
        }

        Ok(outcomes)
    }

    fn prepare_block<'a>(
        &self,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
    ) -> Vec<(&'a Transaction, TransactionOutcome)> {
        let inner = &mut *self.inner.lock();

        let storage = match inner.current_storage {
            Some(ref storage) => storage,
            None => panic!("A StorageAccess should've been set via a call to open_block"),
        };
        let storage = &mut *storage.lock();

        let txs: Vec<_> = transactions.collect();
        let owned_txs: Vec<_> = txs.iter().map(|tx| (*tx).clone()).collect();

        let SortedTxs {
            sorted,
            ..
        } = inner.tx_sorter.sort_txs(&owned_txs);

        let mut tx_n_outcomes: Vec<(&'a Transaction, TransactionOutcome)> = Vec::new();
        let mut remaining_block_space = self.max_body_size;

        for index in sorted {
            let tx = &txs[index].tx;
            if let Some(owner) = inner.tx_owner.get_mut(tx.tx_type()) {
                if remaining_block_space <= tx.size() {
                    break
                }
                storage.create_checkpoint();
                if let Ok(outcome) = owner.execute_transaction(&tx) {
                    storage.discard_checkpoint();
                    tx_n_outcomes.push((tx, outcome));
                    remaining_block_space -= tx.size();
                    continue
                }
                storage.revert_to_the_checkpoint()
            }
        }
        tx_n_outcomes
    }

    fn close_block(&self) -> Result<BlockOutcome, CloseBlockError> {
        let inner = &mut *self.inner.lock();

        let mut events = Vec::new();
        for owner in inner.tx_owner.values_mut() {
            events.extend(owner.block_closed()?.into_iter());
        }
        let (updated_validator_set, updated_consensus_params) = inner.update_chain.update_chain();

        Ok(BlockOutcome {
            updated_validator_set,
            updated_consensus_params,
            events,
        })
    }
}

impl TxFilter for Coordinator {
    fn check_transaction(&self, tx: &Transaction) -> Result<(), ErrorCode> {
        let inner = &mut *self.inner.lock();

        match inner.tx_owner.get(tx.tx_type()) {
            Some(owner) => owner.check_transaction(tx),
            // FIXME: proper error code management is required
            None => Err(ErrorCode::MAX),
        }
    }

    fn filter_transactions<'a>(
        &self,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> FilteredTxs<'a> {
        let inner = &mut *self.inner.lock();

        let txs: Vec<_> = transactions.collect();
        let owned_txs: Vec<_> = txs.iter().map(|tx| (*tx).clone()).collect();

        let SortedTxs {
            sorted,
            invalid,
        } = inner.tx_sorter.sort_txs(&owned_txs);

        let memory_limit = memory_limit.unwrap_or(usize::MAX);
        let mut memory_usage = 0;
        let size_limit = size_limit.unwrap_or_else(|| txs.len());

        let low_priority = sorted
            .into_iter()
            .map(|i| &txs[i].tx)
            .enumerate()
            .skip_while(|(i, tx)| {
                memory_usage += (*tx).size();
                *i >= size_limit || memory_limit >= memory_usage
            })
            .map(|(_, tx)| tx)
            .collect();

        let invalid = invalid.into_iter().map(|i| &txs[i].tx).collect();

        FilteredTxs {
            invalid,
            low_priority,
        }
    }
}
