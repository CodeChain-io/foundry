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
pub mod test_coordinator;
mod transaction;
pub mod types;
pub mod values;
mod weaver;

pub use crate::app_desc::AppDesc;
use crate::context::StorageAccess;
use crate::engine::{BlockExecutor, ExecutionId, GraphQlHandlerProvider, Initializer, TxFilter};
pub use crate::header::Header;
use crate::module::{
    HandleCrimes, HandleGraphQlRequest, InitChain, InitGenesis, SessionId, SortedTxs, Stateful, TxOwner, TxSorter,
    UpdateChain,
};
pub use crate::transaction::{Transaction, TransactionWithMetadata, TxOrigin};
use crate::types::{
    BlockOutcome, CloseBlockError, ErrorCode, ExecuteTransactionError, FilteredTxs, HeaderError, TransactionOutcome,
    VerifiedCrime,
};
use crate::weaver::Weaver;
use cmodule::sandbox::Sandbox;
use ctypes::StorageId;
use ctypes::{CompactValidatorSet, ConsensusParams};
use once_cell::sync::OnceCell;
use parking_lot::{Mutex, RwLock};
use remote_trait_object::{Service, ServiceRef};
use std::collections::HashMap;
use std::mem;
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
    ((Included(0), Unbounded), "handle-graphql-request"),
];

type SessionSlot = u128;

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.
pub struct Coordinator {
    /// The maximum block size.
    max_body_size: OnceCell<usize>,

    /// Currently active sessions represented as bits set.
    sessions: RwLock<Vec<SessionSlot>>,

    /// The key services from modules for implementing a chain.
    services: Services,

    /// List of `Sandbox`es of the modules constituting the current application.
    _sandboxes: Vec<Box<dyn Sandbox>>,
}

const SESSION_BITS_PER_SLOT: usize = mem::size_of::<SessionSlot>() * 8;

impl Coordinator {
    pub fn from_app_desc(app_desc: &AppDesc) -> anyhow::Result<Coordinator> {
        cmodule::init_modules();

        let weaver = Weaver::new();
        let (sandboxes, mut services) = weaver.weave(app_desc)?;

        services.genesis_config = app_desc
            .modules
            .iter()
            .map(|(name, setup)| ((**name).clone(), serde_cbor::to_vec(&setup.genesis_config).unwrap()))
            .collect();

        Ok(Coordinator {
            services,
            _sandboxes: sandboxes,
            max_body_size: Default::default(),
            sessions: RwLock::new(vec![0]),
        })
    }

    pub fn max_body_size(&self) -> usize {
        *self.max_body_size.get().expect("the max_body_size is not set yet")
    }

    fn new_session(&self, storage: &mut dyn StorageAccess) -> SessionId {
        let mut sessions = self.sessions.write();
        let (index, bit) = sessions
            .iter()
            .enumerate()
            .find_map(|(i, &bits)| {
                if bits == SessionSlot::MAX {
                    None
                } else {
                    Some((i, bits.trailing_ones()))
                }
            })
            .unwrap_or_else(|| {
                sessions.push(0);
                (sessions.len() - 1, 0)
            });

        sessions[index] |= 1 << bit;
        let session_id = bit + (SESSION_BITS_PER_SLOT * index) as SessionId;

        let mut statefuls = self.services.stateful.lock();
        for (storage_id, (_, stateful)) in statefuls.iter_mut().enumerate() {
            let sub_storage = storage.sub_storage(storage_id as StorageId);
            stateful.new_session(session_id, ServiceRef::create_export(sub_storage));
        }

        session_id
    }

    fn end_session(&self, session_id: SessionId) {
        let mut statefuls = self.services.stateful.lock();
        for (_, ref mut stateful) in statefuls.iter_mut() {
            stateful.end_session(session_id);
        }
        let mut sessions = self.sessions.write();
        let session_id = session_id as usize;
        sessions[session_id / SESSION_BITS_PER_SLOT] &= !(1 << (session_id % SESSION_BITS_PER_SLOT));
    }

    pub fn services(&self) -> &Services {
        &self.services
    }
}

pub struct Services {
    /// List of module name and `Stateful` service pairs in the current app.
    /// The module name is used to keep the index of the corresponding `Stateful`
    /// same across updates, since the index is used as `StorageId`.
    pub stateful: Mutex<Vec<(String, Box<dyn Stateful>)>>,

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

    /// A map from module name to its GraphQL handler
    pub handle_graphqls: Vec<(String, Arc<dyn HandleGraphQlRequest>)>,
}

impl Default for Services {
    fn default() -> Self {
        Self {
            stateful: Mutex::new(Vec::new()),
            init_genesis: Vec::new(),
            genesis_config: Default::default(),
            tx_owner: Default::default(),
            handle_crimes: Box::new(NoOpHandleCrimes) as Box<dyn HandleCrimes>,
            init_chain: Box::new(PanickingInitChain) as Box<dyn InitChain>,
            update_chain: Box::new(NoOpUpdateChain) as Box<dyn UpdateChain>,
            tx_sorter: Box::new(DefaultTxSorter) as Box<dyn TxSorter>,
            handle_graphqls: Default::default(),
        }
    }
}

struct NoOpHandleCrimes;

impl Service for NoOpHandleCrimes {}

impl HandleCrimes for NoOpHandleCrimes {
    fn handle_crimes(&self, _session_id: SessionId, _crimes: &[VerifiedCrime]) {}
}

struct PanickingInitChain;

impl Service for PanickingInitChain {}

impl InitChain for PanickingInitChain {
    fn init_chain(&self, _session_id: SessionId) -> (CompactValidatorSet, ConsensusParams) {
        panic!("There must be a `InitChain` service")
    }
}

struct NoOpUpdateChain;

impl Service for NoOpUpdateChain {}

impl UpdateChain for NoOpUpdateChain {
    fn update_chain(&self, _session_id: SessionId) -> (Option<CompactValidatorSet>, Option<ConsensusParams>) {
        (None, None)
    }
}

struct DefaultTxSorter;

impl Service for DefaultTxSorter {}

impl TxSorter for DefaultTxSorter {
    fn sort_txs(&self, _session_id: SessionId, txs: &[TransactionWithMetadata]) -> SortedTxs {
        SortedTxs {
            invalid: Vec::new(),
            sorted: (0..txs.len()).collect(),
        }
    }
}

impl Initializer for Coordinator {
    fn number_of_sub_storages(&self) -> usize {
        self.services.stateful.lock().len()
    }

    fn initialize_chain(&self, storage: &mut dyn StorageAccess) -> (CompactValidatorSet, ConsensusParams) {
        let services = &self.services;
        let session_id = self.new_session(storage);

        for (ref module, ref init) in services.init_genesis.iter() {
            let config = match services.genesis_config.get(module) {
                Some(value) => value as &[u8],
                None => &[],
            };
            init.init_genesis(session_id, config);
        }

        let (validator_set, params) = services.init_chain.init_chain(session_id);

        self.max_body_size.set(params.max_body_size() as usize).expect("this must be the first assignment");
        self.end_session(session_id);

        (validator_set, params)
    }
}

impl BlockExecutor for Coordinator {
    fn open_block(
        &self,
        storage: &mut dyn StorageAccess,
        header: &Header,
        verified_crimes: &[VerifiedCrime],
    ) -> Result<ExecutionId, HeaderError> {
        let services = &self.services;

        let session_id = self.new_session(storage);

        services.handle_crimes.handle_crimes(session_id, verified_crimes);

        for owner in services.tx_owner.values() {
            owner.block_opened(session_id, header)?;
        }

        Ok(session_id)
    }

    fn execute_transactions(
        &self,
        execution_id: ExecutionId,
        storage: &mut dyn StorageAccess,
        transactions: &[Transaction],
    ) -> Result<Vec<TransactionOutcome>, ExecuteTransactionError> {
        let services = &self.services;

        let mut outcomes = Vec::with_capacity(transactions.len());
        let session_id = execution_id as SessionId;

        for tx in transactions {
            match services.tx_owner.get(tx.tx_type()) {
                Some(owner) => {
                    storage.create_checkpoint();
                    match owner.execute_transaction(session_id, tx) {
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
        execution_id: ExecutionId,
        storage: &mut dyn StorageAccess,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
    ) -> Vec<(&'a Transaction, TransactionOutcome)> {
        let services = &self.services;

        let txs: Vec<_> = transactions.collect();
        let owned_txs: Vec<_> = txs.iter().map(|tx| (*tx).clone()).collect();
        let session_id = execution_id as SessionId;

        let SortedTxs {
            sorted,
            ..
        } = services.tx_sorter.sort_txs(session_id, &owned_txs);

        let mut tx_n_outcomes: Vec<(&'a Transaction, TransactionOutcome)> = Vec::new();
        let mut remaining_block_space = self.max_body_size();

        for index in sorted {
            let tx = &txs[index].tx;
            if let Some(owner) = services.tx_owner.get(tx.tx_type()) {
                if remaining_block_space <= tx.size() {
                    break
                }
                storage.create_checkpoint();
                if let Ok(outcome) = owner.execute_transaction(session_id, &tx) {
                    storage.discard_checkpoint();
                    tx_n_outcomes.push((tx, outcome));
                    remaining_block_space -= tx.size();
                    continue
                }
                storage.revert_to_the_checkpoint();
            }
        }
        tx_n_outcomes
    }

    fn close_block(&self, execution_id: ExecutionId) -> Result<BlockOutcome, CloseBlockError> {
        let services = &self.services;

        let session_id = execution_id as SessionId;
        let mut events = Vec::new();
        for owner in services.tx_owner.values() {
            events.extend(owner.block_closed(session_id)?.into_iter());
        }
        let (updated_validator_set, updated_consensus_params) = services.update_chain.update_chain(session_id);

        self.end_session(session_id);

        Ok(BlockOutcome {
            updated_validator_set,
            updated_consensus_params,
            events,
        })
    }
}

impl TxFilter for Coordinator {
    fn check_transaction(&self, tx: &Transaction) -> Result<(), ErrorCode> {
        let services = &self.services;

        match services.tx_owner.get(tx.tx_type()) {
            Some(owner) => owner.check_transaction(tx),
            // FIXME: proper error code management is required
            None => Err(ErrorCode::MAX),
        }
    }

    fn filter_transactions<'a>(
        &self,
        storage: &mut dyn StorageAccess,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> FilteredTxs<'a> {
        let services = &self.services;

        let txs: Vec<_> = transactions.collect();
        let owned_txs: Vec<_> = txs.iter().map(|tx| (*tx).clone()).collect();

        let session_id = self.new_session(storage);

        let SortedTxs {
            sorted,
            invalid,
        } = services.tx_sorter.sort_txs(session_id, &owned_txs);

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
        self.end_session(session_id);

        FilteredTxs {
            invalid,
            low_priority,
        }
    }
}

impl GraphQlHandlerProvider for Coordinator {
    fn get(&self) -> Vec<(String, Arc<dyn HandleGraphQlRequest>)> {
        self.services.handle_graphqls.to_vec()
    }

    fn new_session_for_query(&self, storage: &mut dyn StorageAccess) -> crate::module::SessionId {
        self.new_session(storage)
    }

    fn end_session_for_query(&self, session: crate::module::SessionId) {
        self.end_session(session)
    }
}
