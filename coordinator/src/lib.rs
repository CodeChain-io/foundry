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
mod linkable;
pub mod module;
mod substorage;
pub mod test_coordinator;
mod transaction;
pub mod types;
pub mod values;
mod weaver;

use crate::app_desc::AppDesc;
use crate::context::{StorageAccess, SubStorageAccess};
use crate::engine::{BlockExecutor, FilteredTxs, Initializer, TxFilter};
pub use crate::header::Header;
use crate::module::{HandleCrimes, InitChain, InitGenesis, SortedTxs, Stateful, TxOwner, TxSorter, UpdateChain};
use crate::substorage::SubStorageView;
pub use crate::transaction::{Transaction, TransactionWithMetadata, TxOrigin};
use crate::types::{BlockOutcome, ErrorCode, VerifiedCrime};
use crate::types::{CloseBlockError, ExecuteTransactionError, HeaderError, TransactionOutcome};
use crate::weaver::Weaver;
use cmodule::sandbox::Sandbox;
use ctypes::StorageId;
use ctypes::{CompactValidatorSet, ConsensusParams};
use parking_lot::Mutex;
use remote_trait_object::{Service, ServiceRef};
use std::collections::{BTreeMap, HashMap};
use std::ops::Bound;
use std::ops::Bound::*;
use std::sync::Arc;

pub(crate) const HOST_ID: &str = "$";

pub(crate) const TX_SERVICES_FOR_HOST: &[&str] = &["tx-owner"];

pub(crate) type Occurrences = (Bound<usize>, Bound<usize>);

pub(crate) static SERVICES_FOR_HOST: &[(Occurrences, &str)] = &[
    ((Included(0), Unbounded), "init-genesis"),
    ((Included(1), Excluded(2)), "init-chain"),
    ((Included(0), Excluded(2)), "update-chain"),
    ((Included(0), Unbounded), "stateful"),
    ((Included(0), Excluded(2)), "tx-sorter"),
    ((Included(0), Excluded(2)), "handle-crimes"),
];

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.
pub struct Coordinator {
    /// List of `Sandbox`es of the modules constituting the current application.
    _sandboxes: Vec<Box<dyn Sandbox>>,

    /// The key services from modules for implementing a chain.
    inner: Mutex<Inner>,
}

impl Coordinator {
    pub fn from_app_desc(app_desc: &str) -> anyhow::Result<Coordinator> {
        cmodule::init_modules();

        let mut app_desc = AppDesc::from_str(app_desc)?;
        // TODO: proper parameter merging must be implemented with actual parameters from configs
        app_desc.merge_params(&BTreeMap::new())?;

        let weaver = Weaver::new();
        let (sandboxes, mut inner) = weaver.weave(&app_desc)?;

        inner.genesis_config = app_desc
            .modules
            .iter()
            .map(|(name, setup)| ((**name).clone(), serde_cbor::to_vec(&setup.genesis_config).unwrap()))
            .collect();

        let inner = Mutex::new(inner);

        Ok(Coordinator {
            inner,
            _sandboxes: sandboxes,
        })
    }
}

struct Inner {
    /// The maximum block size.
    max_body_size: usize,

    /// The current storage set to all `Stateful` modules.
    current_storage: Arc<Mutex<dyn StorageAccess>>,

    /// List of module name and `Stateful` service pairs in the current app.
    /// The module name is used to keep the index of the corresponding `Stateful`
    /// same across updates, since the index is used as `StorageId`.
    pub stateful: Vec<(String, Box<dyn Stateful>)>,

    /// List of module name and its `InitGenesis` pairs.
    pub init_genesis: Vec<(String, Box<dyn InitGenesis>)>,

    /// Per-module genesis config.
    pub genesis_config: HashMap<String, Vec<u8>>,

    /// A map from Tx type to its owner.
    pub tx_owner: HashMap<String, Box<dyn TxOwner>>,

    /// An optional crime handler.
    pub handle_crimes: Box<dyn HandleCrimes>,

    /// A service responsible for initializing the validators and the parameters.
    pub init_chain: Box<dyn InitChain>,

    /// A service responsible for updating the validators and the parameters when closing every block.
    pub update_chain: Box<dyn UpdateChain>,

    /// A service sorting Tx'es in the mempool.
    pub tx_sorter: Box<dyn TxSorter>,
}

impl Inner {
    pub fn new() -> Self {
        Self::default()
    }

    fn set_storage(&mut self, storage: &Arc<Mutex<dyn StorageAccess>>) {
        self.current_storage = Arc::clone(storage);
        for (id, (_name, ref mut stateful)) in self.stateful.iter_mut().enumerate() {
            let substorage: Box<dyn SubStorageAccess> =
                Box::new(SubStorageView::of(id as StorageId, Arc::clone(storage)));
            stateful.set_storage(ServiceRef::create_export(substorage));
        }
    }
}

impl Default for Inner {
    fn default() -> Self {
        Inner {
            current_storage: Arc::new(Mutex::new(NoOpStorage)) as Arc<Mutex<dyn StorageAccess>>,
            max_body_size: 0,
            stateful: Vec::new(),
            init_genesis: Vec::new(),
            genesis_config: Default::default(),
            tx_owner: Default::default(),
            handle_crimes: Box::new(NoOpHandleCrimes) as Box<dyn HandleCrimes>,
            init_chain: Box::new(PanickingInitChain) as Box<dyn InitChain>,
            update_chain: Box::new(NoOpUpdateChain) as Box<dyn UpdateChain>,
            tx_sorter: Box::new(DefaultTxSorter) as Box<dyn TxSorter>,
        }
    }
}

struct NoOpStorage;

impl StorageAccess for NoOpStorage {
    fn get(&self, _storage_id: u16, _key: &dyn AsRef<[u8]>) -> Option<Vec<u8>> {
        Some(Vec::default())
    }

    fn set(&mut self, _storage_id: u16, _key: &dyn AsRef<[u8]>, _value: Vec<u8>) {}

    fn has(&self, _storage_id: u16, _key: &dyn AsRef<[u8]>) -> bool {
        false
    }

    fn remove(&mut self, _storage_id: u16, _key: &dyn AsRef<[u8]>) {}

    fn create_checkpoint(&mut self) {}

    fn revert_to_the_checkpoint(&mut self) {}

    fn discard_checkpoint(&mut self) {}
}

struct NoOpHandleCrimes;

impl Service for NoOpHandleCrimes {}

impl HandleCrimes for NoOpHandleCrimes {
    fn handle_crimes(&mut self, _crimes: &[VerifiedCrime]) {}
}

struct PanickingInitChain;

impl Service for PanickingInitChain {}

impl InitChain for PanickingInitChain {
    fn init_chain(&mut self) -> (CompactValidatorSet, ConsensusParams) {
        panic!("There must be a `InitChain` service")
    }
}

struct NoOpUpdateChain;

impl Service for NoOpUpdateChain {}

impl UpdateChain for NoOpUpdateChain {
    fn update_chain(&mut self) -> (Option<CompactValidatorSet>, Option<ConsensusParams>) {
        (None, None)
    }
}

struct DefaultTxSorter;

impl Service for DefaultTxSorter {}

impl TxSorter for DefaultTxSorter {
    fn sort_txs(&self, txs: &[TransactionWithMetadata]) -> SortedTxs {
        SortedTxs {
            invalid: Vec::new(),
            sorted: (0..txs.len()).collect(),
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

        let (validator_set, params) = inner.init_chain.init_chain();

        inner.max_body_size = params.max_body_size() as usize;

        (validator_set, params)
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

        inner.handle_crimes.handle_crimes(verified_crimes);

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

        let storage = &mut *inner.current_storage.lock();

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

        let storage = &mut *inner.current_storage.lock();

        let txs: Vec<_> = transactions.collect();
        let owned_txs: Vec<_> = txs.iter().map(|tx| (*tx).clone()).collect();

        let SortedTxs {
            sorted,
            ..
        } = inner.tx_sorter.sort_txs(&owned_txs);

        let mut tx_n_outcomes: Vec<(&'a Transaction, TransactionOutcome)> = Vec::new();
        let mut remaining_block_space = inner.max_body_size;

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
