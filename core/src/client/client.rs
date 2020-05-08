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

use super::importer::Importer;
use super::{
    BlockChainClient, BlockChainInfo, BlockChainTrait, BlockProducer, ChainNotify, ClientConfig, DatabaseClient,
    EngineClient, EngineInfo, ImportBlock, ImportResult, StateInfo, StateOrBlock,
};
use crate::block::{Block, ClosedBlock, IsBlock, OpenBlock};
use crate::blockchain::{BlockChain, BlockProvider, BodyProvider, EventProvider, HeaderProvider, TransactionAddress};
use crate::client::{ConsensusClient, SnapshotClient, TermInfo};
use crate::consensus::{ConsensusEngine, EngineError};
use crate::encoded;
use crate::error::{BlockImportError, Error, ImportError, SchemeError};
use crate::event::EventSource;
use crate::miner::{Miner, MinerService};
use crate::scheme::Scheme;
use crate::service::ClientIoMessage;
use crate::transaction::{LocalizedTransaction, PendingTransactions};
use crate::types::{BlockId, BlockStatus, TransactionId, VerificationQueueInfo as BlockQueueInfo};
use ccrypto::BLAKE_NULL_RLP;
use cdb::{new_journaldb, Algorithm, AsHashDB};
use cio::IoChannel;
use ckey::{Address, NetworkId, PlatformAddress};
use coordinator::context::MemPoolAccess;
use coordinator::traits::{BlockExecutor, Initializer};
use coordinator::types::{Event, Transaction};
use cstate::{Metadata, MetadataAddress, NextValidatorSet, StateDB, StateWithCache, TopLevelState, TopStateView};
use ctimer::{TimeoutHandler, TimerApi, TimerScheduleError, TimerToken};
use ctypes::header::Header;
use ctypes::{BlockHash, BlockNumber, CommonParams, CompactValidatorSet, ConsensusParams, TxHash};
use kvdb::{DBTransaction, KeyValueDB};
use merkle_trie::{TrieFactory, TrieMut};
use parking_lot::{Mutex, RwLock, RwLockReadGuard};
use primitives::{Bytes, H256};
use rlp::{Encodable, Rlp};
use std::ops::Range;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::sync::{Arc, Weak};

const MAX_MEM_POOL_SIZE: usize = 4096;

pub struct Client {
    engine: Arc<dyn ConsensusEngine>,

    io_channel: Mutex<IoChannel<ClientIoMessage>>,

    chain: RwLock<BlockChain>,

    /// Client uses this to store blocks, traces, etc.
    db: Arc<dyn KeyValueDB>,

    state_db: RwLock<StateDB>,

    /// List of actors to be notified on certain chain events
    notify: RwLock<Vec<Weak<dyn ChainNotify>>>,

    /// Count of pending transactions in the queue
    queue_transactions: AtomicUsize,

    importer: Importer,

    /// Timer for reseal_min_period on miner client
    reseal_timer: TimerApi,
}

impl Client {
    pub fn try_new<C: 'static + Initializer + BlockExecutor>(
        config: &ClientConfig,
        scheme: &Scheme,
        db: Arc<dyn KeyValueDB>,
        miner: Arc<Miner>,
        coordinator: Arc<C>,
        message_channel: IoChannel<ClientIoMessage>,
        reseal_timer: TimerApi,
    ) -> Result<Arc<Client>, Error> {
        let journal_db = new_journaldb(Arc::clone(&db), Algorithm::Archive, crate::db::COL_STATE);
        let mut state_db = StateDB::new(journal_db);
        if !scheme.check_genesis_root(state_db.as_hashdb()) {
            return Err(SchemeError::InvalidState.into())
        }
        if state_db.is_empty() {
            let (validators, consensus_params) = coordinator.initialize_chain(scheme.app_state.clone());
            state_db = Self::initialize_state(state_db, consensus_params, validators)?;
            let mut batch = DBTransaction::new();
            state_db.journal_under(&mut batch, 0, *scheme.genesis_header().hash())?;
            db.write(batch)?;
        }

        let gb = scheme.genesis_block();
        let chain = BlockChain::new(&gb, db.clone());

        let engine = scheme.engine.clone();

        let importer = Importer::try_new(config, engine.clone(), message_channel.clone(), miner, coordinator)?;

        let client = Arc::new(Client {
            engine,
            io_channel: Mutex::new(message_channel),
            chain: RwLock::new(chain),
            db,
            state_db: RwLock::new(state_db),
            notify: RwLock::new(Vec::new()),
            queue_transactions: AtomicUsize::new(0),
            importer,
            reseal_timer,
        });

        // ensure buffered changes are flushed.
        client.db.flush()?;
        Ok(client)
    }

    /// Returns engine reference.
    pub fn engine(&self) -> &dyn ConsensusEngine {
        &*self.engine
    }

    /// Adds an actor to be notified on certain events
    pub fn add_notify(&self, target: Weak<dyn ChainNotify>) {
        self.notify.write().push(target);
    }

    pub fn new_blocks(&self, imported: &[BlockHash], invalid: &[BlockHash], enacted: &[BlockHash]) {
        self.notify(|notify| notify.new_blocks(imported.to_vec(), invalid.to_vec(), enacted.to_vec()));
    }

    pub fn new_headers(&self, imported: &[BlockHash], enacted: &[BlockHash], new_best_proposal: Option<BlockHash>) {
        self.notify(|notify| {
            notify.new_headers(imported.to_vec(), enacted.to_vec(), new_best_proposal);
        });
    }

    fn notify<F>(&self, f: F)
    where
        F: Fn(&dyn ChainNotify), {
        for np in self.notify.read().iter() {
            if let Some(n) = np.upgrade() {
                f(&*n);
            }
        }
    }

    /// This is triggered by a message coming from a header queue when the header is ready for insertion
    pub fn import_verified_headers(&self) -> usize {
        self.importer.import_verified_headers_from_queue(self)
    }

    /// This is triggered by a message coming from a block queue when the block is ready for insertion
    pub fn import_verified_blocks(&self) -> usize {
        self.importer.import_verified_blocks(self)
    }

    /// This is triggered by a message coming from a engine when a new block should be created
    pub fn update_sealing(&self, parent_block: BlockId, allow_empty_block: bool) {
        self.importer.miner.update_sealing(self, parent_block, allow_empty_block);
    }

    fn block_hash(chain: &BlockChain, id: &BlockId) -> Option<BlockHash> {
        match id {
            BlockId::Hash(hash) => Some(*hash),
            BlockId::Number(number) => chain.block_hash(*number),
            BlockId::Earliest => chain.block_hash(0),
            BlockId::Latest => Some(chain.best_block_hash()),
            BlockId::ParentOfLatest => Some(chain.best_block_header().parent_hash()),
        }
    }

    fn transaction_address(&self, id: &TransactionId) -> Option<TransactionAddress> {
        match id {
            TransactionId::Hash(hash) => self.block_chain().transaction_address(hash),
            TransactionId::Location(id, index) => {
                Self::block_hash(&self.block_chain(), id).map(|hash| TransactionAddress {
                    block_hash: hash,
                    index: *index,
                })
            }
        }
    }

    /// Import transactions from the IO queue
    pub fn import_queued_transactions(&self, transactions: &[Bytes]) -> usize {
        ctrace!(EXTERNAL_TX, "Importing queued");
        self.queue_transactions.fetch_sub(transactions.len(), AtomicOrdering::SeqCst);
        let transactions: Vec<Transaction> =
            transactions.iter().filter_map(|bytes| Rlp::new(bytes).as_val().ok()).collect();
        let results = self.importer.miner.import_external_transactions(self, transactions);
        results.len()
    }

    /// This is triggered by a message coming from the Tendermint engine when a block is committed.
    /// See EngineClient::update_best_as_committed() for details.
    pub fn update_best_as_committed(&self, block_hash: BlockHash) {
        ctrace!(CLIENT, "Update the best block to the hash({}), as requested", block_hash);
        let update_result = {
            let _import_lock = self.importer.import_lock.lock();

            let chain = self.block_chain();
            let mut batch = DBTransaction::new();

            let update_result = chain.update_best_as_committed(&mut batch, block_hash);
            self.db().write(batch).expect("DB flush failed.");
            chain.commit();

            // Clear the state DB cache
            let mut state_db = self.state_db().write();
            state_db.clear_cache();

            update_result
        };

        if update_result.is_none() {
            return
        }

        let enacted = self.importer.extract_enacted(vec![update_result]);
        self.importer.miner.chain_new_blocks(self, &[], &[], &enacted);
        self.new_blocks(&[], &[], &enacted);
    }

    fn initialize_state(
        db: StateDB,
        genesis_consensus_params: ConsensusParams,
        genesis_validators: CompactValidatorSet,
    ) -> Result<StateDB, Error> {
        let root = BLAKE_NULL_RLP;
        let (db, root) = Self::initialize_validator_set(db, root, genesis_validators)?;
        let db = Self::initialize_modules(db, root, genesis_consensus_params)?;

        Ok(db)
    }

    fn initialize_validator_set(
        db: StateDB,
        root: H256,
        genesis_validators: CompactValidatorSet,
    ) -> Result<(StateDB, H256), Error> {
        let mut state = TopLevelState::from_existing(db.clone(&root), root)?;
        let validator_set = NextValidatorSet::from_compact_validator_set(genesis_validators);
        validator_set.save_to_state(&mut state)?;
        let root = state.commit()?;
        Ok((db, root))
    }

    fn initialize_modules(
        mut db: StateDB,
        mut root: H256,
        genesis_consensus_params: ConsensusParams,
    ) -> Result<StateDB, Error> {
        // TODO: remove CommonParams
        let genesis_params = CommonParams::default();
        let global_metadata = Metadata::new(genesis_params, genesis_consensus_params);
        {
            let mut t = TrieFactory::from_existing(db.as_hashdb_mut(), &mut root)?;
            let address = MetadataAddress::new();

            let r = t.insert(&*address, &global_metadata.rlp_bytes());
            debug_assert_eq!(Ok(None), r);
            r?;
        }
        Ok(db)
    }

    fn block_number_ref(&self, id: &BlockId) -> Option<BlockNumber> {
        match id {
            BlockId::Number(number) => Some(*number),
            BlockId::Hash(hash) => self.block_chain().block_number(hash),
            BlockId::Earliest => Some(0),
            BlockId::Latest => Some(self.block_chain().best_block_detail().number),
            BlockId::ParentOfLatest => {
                if self.block_chain().best_block_detail().number == 0 {
                    None
                } else {
                    Some(self.block_chain().best_block_detail().number - 1)
                }
            }
        }
    }

    fn state_info(&self, state: StateOrBlock) -> Option<Box<dyn TopStateView>> {
        Some(match state {
            StateOrBlock::State(state) => state,
            StateOrBlock::Block(id) => Box::new(self.state_at(id)?),
        })
    }

    pub fn state_db(&self) -> &RwLock<StateDB> {
        &self.state_db
    }

    pub fn block_chain(&self) -> RwLockReadGuard<'_, BlockChain> {
        self.chain.read()
    }

    pub fn db(&self) -> &Arc<dyn KeyValueDB> {
        &self.db
    }
}

/// The minimum time between blocks, the miner creates a block when RESEAL_MIN_TIMER is invoked.
/// Do not create a block before RESEAL_MIN_TIMER event.
const RESEAL_MIN_TIMER_TOKEN: TimerToken = 1;

impl TimeoutHandler for Client {
    fn on_timeout(&self, token: TimerToken) {
        match token {
            RESEAL_MIN_TIMER_TOKEN => {
                // Checking self.ready_transactions() for efficiency
                if !self.engine().engine_type().ignore_reseal_min_period() && !self.is_pending_queue_empty() {
                    self.update_sealing(BlockId::Latest, false);
                }
            }
            _ => unreachable!(),
        }
    }
}

impl DatabaseClient for Client {
    fn database(&self) -> Arc<dyn KeyValueDB> {
        Arc::clone(&self.db())
    }
}

impl StateInfo for Client {
    fn state_at(&self, id: BlockId) -> Option<TopLevelState> {
        self.block_header(&id).and_then(|header| {
            let root = header.state_root();
            TopLevelState::from_existing(self.state_db.read().clone(&root), root).ok()
        })
    }
}

impl EngineInfo for Client {
    fn network_id(&self) -> NetworkId {
        self.consensus_params(BlockId::Earliest).expect("Genesis state must exist").network_id()
    }

    fn common_params(&self, block_id: BlockId) -> Option<CommonParams> {
        self.state_info(block_id.into()).map(|state| {
            *state
                .metadata()
                .unwrap_or_else(|err| unreachable!("Unexpected failure. Maybe DB was corrupted: {:?}", err))
                .unwrap()
                .params()
        })
    }

    fn consensus_params(&self, block_id: BlockId) -> Option<ConsensusParams> {
        self.state_info(block_id.into()).map(|state| {
            *state
                .metadata()
                .unwrap_or_else(|err| unreachable!("Unexpected failure. Maybe DB was corrupted: {:?}", err))
                .unwrap()
                .consensus_params()
        })
    }

    fn metadata_seq(&self, block_id: BlockId) -> Option<u64> {
        self.state_info(block_id.into()).map(|state| {
            state
                .metadata()
                .unwrap_or_else(|err| unreachable!("Unexpected failure. Maybe DB was corrupted: {:?}", err))
                .unwrap()
                .seq()
        })
    }

    fn possible_authors(&self, block_number: Option<u64>) -> Result<Option<Vec<PlatformAddress>>, EngineError> {
        let network_id = self.network_id();
        if block_number == Some(0) {
            let genesis_author = self.block_header(&0.into()).expect("genesis block").author();
            return Ok(Some(vec![PlatformAddress::new_v1(network_id, genesis_author)]))
        }
        let addresses = self.engine().possible_authors(block_number)?;
        Ok(addresses.map(|addresses| {
            addresses.into_iter().map(|address| PlatformAddress::new_v1(network_id, address)).collect()
        }))
    }
}

impl EngineClient for Client {
    /// Make a new block and seal it.
    fn update_sealing(&self, parent_block: BlockId, allow_empty_block: bool) {
        match self.io_channel.lock().send(ClientIoMessage::NewBlockRequired {
            parent_block,
            allow_empty_block,
        }) {
            Ok(_) => {}
            Err(e) => {
                cdebug!(CLIENT, "Error while triggering block creation: {}", e);
            }
        }
    }

    /// Update the best block as the given block hash.
    ///
    /// Used in Tendermint, when going to the commit step.
    fn update_best_as_committed(&self, block_hash: BlockHash) {
        ctrace!(ENGINE, "Requesting a best block update (block hash: {})", block_hash);
        match self.io_channel.lock().send(ClientIoMessage::UpdateBestAsCommitted(block_hash)) {
            Ok(_) => {}
            Err(e) => {
                cerror!(CLIENT, "Error while triggering the best block update: {}", e);
            }
        }
    }

    fn get_kvdb(&self) -> Arc<dyn KeyValueDB> {
        self.db.clone()
    }
}

impl ConsensusClient for Client {}

impl BlockChainTrait for Client {
    fn chain_info(&self) -> BlockChainInfo {
        self.block_chain().chain_info()
    }

    fn block_header(&self, id: &BlockId) -> Option<encoded::Header> {
        let chain = self.block_chain();

        Self::block_hash(&chain, id).and_then(|hash| chain.block_header_data(&hash))
    }

    fn best_block_header(&self) -> encoded::Header {
        self.block_chain().best_block_header()
    }

    fn best_header(&self) -> encoded::Header {
        self.block_chain().best_header()
    }

    fn best_proposal_header(&self) -> encoded::Header {
        self.block_chain().best_proposal_header()
    }

    fn block(&self, id: &BlockId) -> Option<encoded::Block> {
        let chain = self.block_chain();

        Self::block_hash(&chain, id).and_then(|hash| chain.block(&hash))
    }

    fn transaction_block(&self, id: &TransactionId) -> Option<BlockHash> {
        self.transaction_address(id).map(|addr| addr.block_hash)
    }
}

impl ImportBlock for Client {
    fn import_block(&self, bytes: Bytes) -> Result<BlockHash, BlockImportError> {
        use crate::verification::queue::kind::blocks::Unverified;
        use crate::verification::queue::kind::BlockLike;

        let unverified = Unverified::new(bytes);
        {
            if self.block_chain().is_known(&unverified.hash()) {
                return Err(BlockImportError::Import(ImportError::AlreadyInChain))
            }
        }
        Ok(self.importer.block_queue.import(unverified)?)
    }

    fn import_header(&self, unverified: Header) -> Result<BlockHash, BlockImportError> {
        if self.block_chain().is_known_header(&unverified.hash()) {
            return Err(BlockImportError::Import(ImportError::AlreadyInChain))
        }
        Ok(self.importer.header_queue.import(unverified)?)
    }

    fn import_trusted_header(&self, header: Header) -> Result<BlockHash, BlockImportError> {
        if self.block_chain().is_known_header(&header.hash()) {
            return Err(BlockImportError::Import(ImportError::AlreadyInChain))
        }
        let import_lock = self.importer.import_lock.lock();
        self.importer.import_trusted_header(&header, self, &import_lock);
        Ok(header.hash())
    }

    fn import_trusted_block(&self, block: &Block) -> Result<BlockHash, BlockImportError> {
        if self.block_chain().is_known(&block.header.hash()) {
            return Err(BlockImportError::Import(ImportError::AlreadyInChain))
        }
        let import_lock = self.importer.import_lock.lock();
        self.importer.import_trusted_block(block, self, &import_lock);
        Ok(block.header.hash())
    }

    fn force_update_best_block(&self, hash: &BlockHash) {
        self.importer.force_update_best_block(hash, self)
    }

    fn import_generated_block(&self, block: &ClosedBlock) -> ImportResult {
        let h = block.header().hash();
        let update_result = {
            // scope for self.import_lock
            let import_lock = self.importer.import_lock.lock();

            let number = block.header().number();
            let block_data = block.rlp_bytes();
            let header = block.header();

            self.importer.import_verified_headers(vec![header], self, &import_lock);

            let update_result = self.importer.commit_block(block, header, &block_data, self);
            cinfo!(CLIENT, "Imported closed block #{} ({})", number, h);
            update_result
        };
        let enacted = self.importer.extract_enacted(vec![update_result]);
        self.importer.miner.chain_new_blocks(self, &[h], &[], &enacted);
        self.new_blocks(&[h], &[], &enacted);
        self.db().flush().expect("DB flush failed.");
        Ok(h)
    }

    fn set_min_timer(&self) {
        self.reseal_timer.cancel(RESEAL_MIN_TIMER_TOKEN).expect("Reseal min timer clear succeeds");
        match self
            .reseal_timer
            .schedule_once(self.importer.miner.get_options().reseal_min_period, RESEAL_MIN_TIMER_TOKEN)
        {
            Ok(_) => {}
            Err(TimerScheduleError::TokenAlreadyScheduled) => {
                // Since set_min_timer could be called in multi thread, ignore the TokenAlreadyScheduled error
            }
            Err(err) => unreachable!("Reseal min timer should not fail but failed with {:?}", err),
        }
    }
}

impl MemPoolAccess for Client {
    fn inject_transactions(&self, transactions: Vec<Transaction>) -> Vec<Result<TxHash, String>> {
        transactions
            .into_iter()
            .map(|tx| {
                // FIXME: tx_hash is calculated even if failed to queue
                let tx_hash = tx.hash();
                self.queue_own_transaction(tx).map(|_| tx_hash).map_err(|e| format!("{}", e))
            })
            .collect()
    }
}

impl BlockChainClient for Client {
    fn queue_info(&self) -> BlockQueueInfo {
        self.importer.block_queue.queue_info()
    }

    /// Import own transaction
    fn queue_own_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        self.importer.miner.import_own_transaction(self, transaction)
    }

    fn queue_transactions(&self, transactions: Vec<Bytes>) {
        let queue_size = self.queue_transactions.load(AtomicOrdering::Relaxed);
        ctrace!(EXTERNAL_TX, "Queue size: {}", queue_size);
        if queue_size > MAX_MEM_POOL_SIZE {
            cwarn!(EXTERNAL_TX, "Ignoring {} transactions: queue is full", transactions.len());
        } else {
            let len = transactions.len();
            match self.io_channel.lock().send(ClientIoMessage::NewTransactions(transactions)) {
                Ok(_) => {
                    self.queue_transactions.fetch_add(len, AtomicOrdering::SeqCst);
                }
                Err(e) => {
                    cwarn!(EXTERNAL_TX, "Ignoring {} transactions: error queueing: {}", len, e);
                }
            }
        }
    }

    fn delete_all_pending_transactions(&self) {
        self.importer.miner.delete_all_pending_transactions();
    }

    fn ready_transactions(&self, range: Range<u64>) -> PendingTransactions {
        let params =
            self.consensus_params(BlockId::Latest).expect("Consensus params of the latest block always exists");

        self.importer.miner.ready_transactions(params.max_body_size(), params.max_body_size(), range)
    }

    fn count_pending_transactions(&self, range: Range<u64>) -> usize {
        self.importer.miner.count_pending_transactions(range)
    }

    fn is_pending_queue_empty(&self) -> bool {
        self.importer.miner.status().transactions_in_pending_queue == 0
    }

    fn block_number(&self, id: &BlockId) -> Option<BlockNumber> {
        self.block_number_ref(&id)
    }

    fn block_body(&self, id: &BlockId) -> Option<encoded::Body> {
        let chain = self.block_chain();

        Self::block_hash(&chain, id).and_then(|hash| chain.block_body(&hash))
    }

    fn block_status(&self, id: &BlockId) -> BlockStatus {
        let chain = self.block_chain();
        match Self::block_hash(&chain, id) {
            Some(ref hash) if chain.is_known(hash) => BlockStatus::InChain,
            Some(hash) => self.importer.block_queue.status(&hash),
            None => BlockStatus::Unknown,
        }
    }

    fn block_hash(&self, id: &BlockId) -> Option<BlockHash> {
        let chain = self.block_chain();
        Self::block_hash(&chain, id)
    }

    fn transaction(&self, id: &TransactionId) -> Option<LocalizedTransaction> {
        let chain = self.block_chain();
        self.transaction_address(id).and_then(|address| chain.transaction(&address))
    }

    fn events_by_tx_hash(&self, hash: &TxHash) -> Vec<Event> {
        let chain = self.block_chain();
        let source = EventSource::Transaction(*hash);
        chain.events(&source)
    }

    fn events_by_block_hash(&self, hash: &BlockHash) -> Vec<Event> {
        let chain = self.block_chain();
        let source = EventSource::Block(*hash);
        chain.events(&source)
    }
}

impl TermInfo for Client {
    fn last_term_finished_block_num(&self, id: BlockId) -> Option<BlockNumber> {
        self.state_at(id)
            .map(|state| state.metadata().unwrap().expect("Metadata always exist"))
            .map(|metadata| metadata.last_term_finished_block_num())
    }

    fn current_term_id(&self, id: BlockId) -> Option<u64> {
        self.state_at(id)
            .map(|state| state.metadata().unwrap().expect("Metadata always exist"))
            .map(|metadata| metadata.current_term_id())
    }
}

impl BlockProducer for Client {
    fn prepare_open_block(&self, parent_block_id: BlockId, author: Address, extra_data: Bytes) -> OpenBlock<'_> {
        let engine = &*self.engine;
        let chain = self.block_chain();
        let parent_hash = self.block_hash(&parent_block_id).expect("parent exist always");
        let parent_header = chain.block_header(&parent_hash).expect("parent exist always");

        OpenBlock::try_new(
            engine,
            self.state_db.read().clone(&parent_header.state_root()),
            &parent_header,
            author,
            extra_data,
        ).expect("OpenBlock::new only fails if parent state root invalid; state root of best block's header is never invalid; qed")
    }
}

impl SnapshotClient for Client {
    fn notify_snapshot(&self, id: BlockId) {
        if let Some(header) = self.block_header(&id) {
            self.engine.send_snapshot_notify(header.hash())
        }
    }
}
