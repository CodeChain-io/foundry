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

use super::mem_pool::{Error as MemPoolError, MemPool};
use super::mem_pool_types::{MemPoolInput, TxOrigin};
use super::{fetch_account_creator, MinerService, TransactionImportResult};
use crate::account_provider::{AccountProvider, Error as AccountProviderError};
use crate::block::{ClosedBlock, IsBlock};
use crate::client::{
    AccountData, BlockChainTrait, BlockProducer, Client, EngineInfo, ImportBlock, MiningBlockChainClient, TermInfo,
};
use crate::consensus::{ConsensusEngine, EngineType};
use crate::error::Error;
use crate::scheme::Scheme;
use crate::transaction::{PendingVerifiedTransactions, UnverifiedTransaction, VerifiedTransaction};
use crate::types::TransactionId;
use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::engine::{BlockExecutor, TxFilter};
use cstate::{TopLevelState, TopStateView};
use ctypes::errors::HistoryError;
use ctypes::transaction::Transaction;
use ctypes::{BlockHash, BlockId, TransactionIndex};
use kvdb::KeyValueDB;
use parking_lot::{Mutex, RwLock};
use primitives::Bytes;
use rlp::Encodable;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::convert::TryInto;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Configures the behaviour of the miner.
#[derive(Debug, PartialEq)]
pub struct MinerOptions {
    /// Reseal on receipt of new external transactions.
    pub reseal_on_external_transaction: bool,
    /// Reseal on receipt of new local transactions.
    pub reseal_on_own_transaction: bool,
    /// Minimum period between transaction-inspired reseals.
    pub reseal_min_period: Duration,
    /// Maximum size of the mem pool.
    pub mem_pool_size: usize,
    /// Maximum memory usage of transactions in the queue (current / future).
    pub mem_pool_memory_limit: Option<usize>,
    /// A value which is used to check whether a new transaciton can replace a transaction in the memory pool with the same signer and seq.
    /// If the fee of the new transaction is `new_fee` and the fee of the transaction in the memory pool is `old_fee`,
    /// then `new_fee > old_fee + old_fee >> mem_pool_fee_bump_shift` should be satisfied to replace.
    /// Local transactions ignore this option.
    pub mem_pool_fee_bump_shift: usize,
}

impl Default for MinerOptions {
    fn default() -> Self {
        MinerOptions {
            reseal_on_external_transaction: true,
            reseal_on_own_transaction: true,
            reseal_min_period: Duration::from_secs(2),
            mem_pool_size: 8192,
            mem_pool_memory_limit: Some(2 * 1024 * 1024),
            mem_pool_fee_bump_shift: 3,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AuthoringParams {
    pub author: Public,
    pub extra_data: Bytes,
}

pub struct Miner {
    mem_pool: Arc<RwLock<MemPool>>,
    next_allowed_reseal: NextAllowedReseal,
    params: Params,
    engine: Arc<dyn ConsensusEngine>,
    options: MinerOptions,

    sealing_enabled: AtomicBool,

    accounts: Arc<AccountProvider>,

    _block_executor: Arc<dyn BlockExecutor>,
}

struct Params {
    params: RwLock<AuthoringParams>,
}

impl Params {
    pub fn new(params: AuthoringParams) -> Self {
        Self {
            params: RwLock::new(params),
        }
    }

    pub fn get(&self) -> AuthoringParams {
        self.params.read().clone()
    }

    pub fn apply<F>(&self, f: F)
    where
        F: FnOnce(&mut AuthoringParams) -> (), {
        let mut params = self.params.write();
        f(&mut params);
    }
}

struct NextAllowedReseal {
    instant: Mutex<Instant>,
}

impl NextAllowedReseal {
    pub fn new(instant: Instant) -> Self {
        Self {
            instant: Mutex::new(instant),
        }
    }

    pub fn get(&self) -> Instant {
        *self.instant.lock()
    }

    pub fn set(&self, instant: Instant) {
        *self.instant.lock() = instant;
    }
}

impl Miner {
    pub fn new<C: 'static + BlockExecutor + TxFilter>(
        options: MinerOptions,
        scheme: &Scheme,
        accounts: Arc<AccountProvider>,
        db: Arc<dyn KeyValueDB>,
        block_executor: Arc<C>,
    ) -> Arc<Self> {
        Arc::new(Self::new_raw(options, scheme, accounts, db, block_executor))
    }

    pub fn with_scheme_for_test<C: 'static + BlockExecutor + TxFilter>(
        scheme: &Scheme,
        db: Arc<dyn KeyValueDB>,
        coordinator: Arc<C>,
    ) -> Self {
        Self::new_raw(Default::default(), scheme, AccountProvider::transient_provider(), db, coordinator)
    }

    fn new_raw<C: 'static + BlockExecutor + TxFilter>(
        options: MinerOptions,
        scheme: &Scheme,
        accounts: Arc<AccountProvider>,
        db: Arc<dyn KeyValueDB>,
        coordinator: Arc<C>,
    ) -> Self {
        let mem_limit = options.mem_pool_memory_limit.unwrap_or_else(usize::max_value);
        let mem_pool = Arc::new(RwLock::new(MemPool::with_limits(
            options.mem_pool_size,
            mem_limit,
            options.mem_pool_fee_bump_shift,
            db,
            coordinator.clone(),
        )));

        Self {
            mem_pool,
            next_allowed_reseal: NextAllowedReseal::new(Instant::now()),
            params: Params::new(AuthoringParams::default()),
            engine: scheme.engine.clone(),
            options,
            sealing_enabled: AtomicBool::new(true),
            accounts,
            _block_executor: coordinator,
        }
    }

    pub fn recover_from_db(&self, client: &Client) {
        self.mem_pool.write().recover_from_db(client);
    }

    pub fn get_options(&self) -> &MinerOptions {
        &self.options
    }

    fn add_transactions_to_pool<C: AccountData + BlockChainTrait + EngineInfo>(
        &self,
        client: &C,
        transactions: Vec<UnverifiedTransaction>,
        default_origin: TxOrigin,
        mem_pool: &mut MemPool,
    ) -> Vec<Result<TransactionImportResult, Error>> {
        let best_header = client.best_block_header().decode();
        let current_block_number = client.chain_info().best_block_number;
        let current_timestamp = client.chain_info().best_block_timestamp;
        let mut inserted = Vec::with_capacity(transactions.len());
        let mut to_insert = Vec::new();
        let mut tx_hashes = Vec::new();

        let intermediate_results: Vec<Result<(), Error>> = transactions
            .into_iter()
            .map(|tx| {
                let hash = tx.hash();
                // FIXME: Refactoring is needed. recover_public is calling in verify_transaction_unordered.
                let signer_public = tx.signer_public();

                let origin = if self.accounts.has_account(&signer_public).unwrap_or_default() {
                    TxOrigin::Local
                } else {
                    default_origin
                };

                if client.transaction_block(&TransactionId::Hash(hash)).is_some() {
                    cdebug!(MINER, "Rejected transaction {:?}: already in the blockchain", hash);
                    return Err(HistoryError::TransactionAlreadyImported.into())
                }
                let tx: VerifiedTransaction =
                    tx.verify_basic().map_err(From::from).and_then(|_| tx.try_into()).map_err(|e: Error| {
                        cdebug!(MINER, "Rejected transaction {:?} with error {:?}", hash, e);
                        e
                    })?;

                let tx_hash = tx.hash();

                to_insert.push(MemPoolInput::new(tx, origin));
                tx_hashes.push(tx_hash);
                Ok(())
            })
            .collect();

        let block_id = BlockId::Hash(best_header.hash());
        let fetch_account = fetch_account_creator(client, block_id);

        let insertion_results = mem_pool.add(to_insert, current_block_number, current_timestamp, &fetch_account);

        debug_assert_eq!(insertion_results.len(), intermediate_results.iter().filter(|r| r.is_ok()).count());
        let mut insertion_results_index = 0;
        intermediate_results
            .into_iter()
            .map(|res| match res {
                Err(e) => Err(e),
                Ok(()) => {
                    let idx = insertion_results_index;
                    let result = insertion_results[idx].clone().map_err(MemPoolError::into_core_error)?;
                    inserted.push(tx_hashes[idx]);
                    insertion_results_index += 1;
                    Ok(result)
                }
            })
            .collect()
    }

    pub fn delete_all_pending_transactions(&self) {
        let mut mem_pool = self.mem_pool.write();
        mem_pool.remove_all();
    }

    /// Prepares new block for sealing including top transactions from queue and seal it.
    fn prepare_and_seal_block<C: AccountData + BlockChainTrait + BlockProducer + EngineInfo + TermInfo>(
        &self,
        parent_block_id: BlockId,
        chain: &C,
    ) -> Result<Option<ClosedBlock>, Error> {
        let (transactions, mut open_block, block_number, block_tx_signer, block_tx_seq) = {
            ctrace!(MINER, "prepare_block: No existing work - making new block");
            let params = self.params.get();
            let open_block = chain.prepare_open_block(parent_block_id, params.author, params.extra_data);
            let (block_number, parent_hash) = {
                let header = open_block.block().header();
                let block_number = header.number();
                let parent_hash = *header.parent_hash();
                (block_number, parent_hash)
            };
            let max_body_size = chain.consensus_params(parent_hash.into()).unwrap().max_body_size() as usize;
            const DEFAULT_RANGE: Range<u64> = 0..u64::MAX;

            // NOTE: This lock should be acquired after `prepare_open_block` to prevent deadlock
            let mem_pool = self.mem_pool.read();

            let mut transactions = Vec::default();
            let (block_tx_signer, block_tx_seq) =
                if let Some(action) = self.engine.open_block_action(open_block.block())? {
                    ctrace!(MINER, "Enqueue a transaction to open block");
                    // TODO: This generates a new random account to make the transaction.
                    // It should use the block signer.
                    let tx_signer = Private::random();
                    let seq = open_block.state().seq(&tx_signer.public_key())?;
                    let tx = Transaction {
                        network_id: chain.network_id(),
                        action,
                        seq,
                        fee: 0,
                    };
                    let verified_tx = VerifiedTransaction::new_with_sign(tx, &tx_signer);
                    transactions.push(verified_tx);
                    (Some(tx_signer), Some(seq + 1))
                } else {
                    (None, None)
                };
            let open_transaction_size = transactions.iter().map(|tx| tx.rlp_bytes().len()).sum();
            assert!(max_body_size > open_transaction_size);
            let mut pending_transactions =
                mem_pool.top_transactions(max_body_size - open_transaction_size, DEFAULT_RANGE).transactions;
            transactions.append(&mut pending_transactions);

            (transactions, open_block, block_number, block_tx_signer, block_tx_seq)
        };

        let parent_header = {
            let parent_hash = open_block.header().parent_hash();
            chain.block_header(&BlockId::Hash(*parent_hash)).expect("Parent header MUST exist")
        };

        assert!(self.engine.seals_internally(), "If a signer is not prepared, prepare_block should not be called");
        let seal = self.engine.generate_seal(None, &parent_header.decode());
        if let Some(seal_bytes) = seal.seal_fields() {
            open_block.seal(self.engine.borrow(), seal_bytes).expect("Sealing always success");
        } else {
            return Ok(None)
        }

        let mut invalid_transactions = Vec::new();

        let mut tx_count: usize = 0;
        let tx_total = transactions.len();
        let mut invalid_tx_users = HashSet::new();

        for tx in transactions {
            let signer_public = tx.signer_public();
            if invalid_tx_users.contains(&signer_public) {
                // The previous transaction has failed
                continue
            }

            let hash = tx.hash();
            let start = Instant::now();
            let transaction_index = tx_count as TransactionIndex;
            // Check whether transaction type is allowed for sender
            let result = open_block.push_transaction(tx, parent_header.number(), transaction_index);

            match result {
                // already have transaction - ignore
                Err(Error::History(HistoryError::TransactionAlreadyImported)) => {}
                Err(e) => {
                    invalid_tx_users.insert(signer_public);
                    invalid_transactions.push(hash);
                    cinfo!(
                        MINER,
                        "Error adding transaction to block: number={}. tx_hash={:?}, Error: {:?}",
                        block_number,
                        hash,
                        e
                    );
                }
                Ok(()) => {
                    let took = start.elapsed();
                    ctrace!(MINER, "Adding transaction {:?} took {:?}", hash, took);
                    tx_count += 1;
                } // imported ok
            }
        }
        cdebug!(MINER, "Pushed {}/{} transactions", tx_count, tx_total);

        let actions = self.engine.close_block_actions(open_block.block()).map_err(|e| {
            warn!("Encountered error on closing the block: {}", e);
            e
        })?;
        if !actions.is_empty() {
            ctrace!(MINER, "Enqueue {} transactions to close block", actions.len());
            // TODO: This generates a new random account to make the transaction.
            // It should use the block signer.
            let tx_signer = block_tx_signer.unwrap_or_else(Private::random);
            let mut seq = block_tx_seq.map(Ok).unwrap_or_else(|| open_block.state().seq(&tx_signer.public_key()))?;
            for (index, action) in actions.into_iter().enumerate() {
                let transaction_index = (tx_count + index) as TransactionIndex;
                let tx = Transaction {
                    network_id: chain.network_id(),
                    action,
                    seq,
                    fee: 0,
                };
                seq += 1;
                let tx = VerifiedTransaction::new_with_sign(tx, &tx_signer);
                // TODO: The current code can insert more transactions than size limit.
                // It should be fixed to pre-calculate the maximum size of the close transactions and prevent the size overflow.
                open_block.push_transaction(tx, parent_header.number(), transaction_index)?;
            }
        }
        let block = open_block.close()?;

        let block_id = {
            let best_block_hash = chain.chain_info().best_block_hash;
            BlockId::Hash(best_block_hash)
        };
        let fetch_seq = |p: &Public| chain.seq(p, block_id).expect("Read from best block");
        {
            let mut mem_pool = self.mem_pool.write();
            mem_pool.remove(
                &invalid_transactions,
                &fetch_seq,
                chain.chain_info().best_block_number,
                chain.chain_info().best_block_timestamp,
            );
        }
        Ok(Some(block))
    }

    /// Attempts to perform internal sealing (one that does not require work) and handles the result depending on the type of Seal.
    fn import_block_internally<C>(&self, chain: &C, block: ClosedBlock) -> bool
    where
        C: BlockChainTrait + ImportBlock, {
        assert!(self.engine.seals_internally());

        if self.engine.is_proposal(block.header()) {
            self.engine.proposal_generated(&block);
        }

        chain.import_generated_block(&block).is_ok()
    }

    /// Are we allowed to do a non-mandatory reseal?
    fn transaction_reseal_allowed(&self) -> bool {
        self.sealing_enabled.load(Ordering::Relaxed) && (Instant::now() > self.next_allowed_reseal.get())
    }
}

impl MinerService for Miner {
    type State = TopLevelState;

    fn num_pending_transactions(&self) -> usize {
        self.mem_pool.read().num_pending_transactions()
    }

    fn authoring_params(&self) -> AuthoringParams {
        self.params.get()
    }

    fn set_author(&self, pubkey: Public) -> Result<(), AccountProviderError> {
        self.params.apply(|params| params.author = pubkey);

        if self.engine_type().need_signer_key() {
            ctrace!(MINER, "Set author to {:?}", pubkey);
            // Sign test message
            self.accounts.get_unlocked_account(&pubkey)?.sign(&Default::default())?;
            self.engine.set_signer(Arc::clone(&self.accounts), pubkey);
        }
        Ok(())
    }

    fn get_author(&self) -> Public {
        self.params.get().author
    }

    fn set_extra_data(&self, extra_data: Bytes) {
        self.params.apply(|params| params.extra_data = extra_data);
    }

    fn transactions_limit(&self) -> usize {
        self.mem_pool.read().limit()
    }

    fn set_transactions_limit(&self, limit: usize) {
        self.mem_pool.write().set_limit(limit)
    }

    fn chain_new_blocks<C>(&self, chain: &C, _imported: &[BlockHash], _invalid: &[BlockHash], _enacted: &[BlockHash])
    where
        C: AccountData + BlockChainTrait + BlockProducer + EngineInfo + ImportBlock, {
        ctrace!(MINER, "chain_new_blocks");

        {
            let block_id = {
                let current_block_hash = chain.chain_info().best_block_hash;
                BlockId::Hash(current_block_hash)
            };
            let fetch_account = fetch_account_creator(chain, block_id);
            let current_block_number = chain.chain_info().best_block_number;
            let current_timestamp = chain.chain_info().best_block_timestamp;
            let mut mem_pool = self.mem_pool.write();
            mem_pool.remove_old(&fetch_account, current_block_number, current_timestamp);
        }

        chain.set_min_timer();
    }

    fn engine_type(&self) -> EngineType {
        self.engine.engine_type()
    }

    fn update_sealing<C>(&self, chain: &C, parent_block: BlockId, allow_empty_block: bool)
    where
        C: AccountData + BlockChainTrait + BlockProducer + EngineInfo + ImportBlock + TermInfo, {
        ctrace!(MINER, "update_sealing: preparing a block");

        let block = match self.prepare_and_seal_block(parent_block, chain) {
            Ok(Some(block)) => {
                if !allow_empty_block && block.block().transactions().is_empty() {
                    ctrace!(MINER, "update_sealing: block is empty, and allow_empty_block is false");
                    return
                }
                block
            }
            Ok(None) => {
                ctrace!(MINER, "update_sealing: cannot prepare block");
                return
            }
            Err(err) => {
                ctrace!(MINER, "update_sealing: cannot prepare block: {:?}", err);
                return
            }
        };

        if std::env::var("RUN_ON_TEST").is_ok() {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).expect("There is no time machine.").as_secs();
            if block.header().timestamp() > now {
                let delta = block.header().timestamp() - now;
                std::thread::sleep(std::time::Duration::from_secs(delta));
            }
        }

        if self.engine.seals_internally() {
            ctrace!(MINER, "update_sealing: engine indicates internal sealing");
            if self.import_block_internally(chain, block) {
                ctrace!(MINER, "update_sealing: imported internally closed block");
            }
        } else {
            ctrace!(MINER, "update_sealing: engine is not keen to seal internally right now");
            return
        }

        // Sealing successful
        self.next_allowed_reseal.set(Instant::now() + self.options.reseal_min_period);
        chain.set_min_timer();
    }

    fn import_external_transactions<C: MiningBlockChainClient + EngineInfo + TermInfo>(
        &self,
        client: &C,
        transactions: Vec<UnverifiedTransaction>,
    ) -> Vec<Result<TransactionImportResult, Error>> {
        ctrace!(EXTERNAL_TX, "Importing external transactions");
        let results = {
            let mut mem_pool = self.mem_pool.write();
            self.add_transactions_to_pool(client, transactions, TxOrigin::External, &mut mem_pool)
        };

        if !results.is_empty()
            && self.options.reseal_on_external_transaction
            && self.transaction_reseal_allowed()
            && !self.engine_type().ignore_reseal_on_transaction()
        {
            // ------------------------------------------------------------------
            // | NOTE Code below requires mem_pool and sealing_queue locks.     |
            // | Make sure to release the locks before calling that method.     |
            // ------------------------------------------------------------------
            self.update_sealing(client, BlockId::Latest, false);
        }
        results
    }

    fn import_own_transaction<C: MiningBlockChainClient + EngineInfo + TermInfo>(
        &self,
        chain: &C,
        tx: VerifiedTransaction,
    ) -> Result<TransactionImportResult, Error> {
        ctrace!(OWN_TX, "Importing transaction: {:?}", tx);

        let imported = {
            // Be sure to release the lock before we call prepare_work_sealing
            let mut mem_pool = self.mem_pool.write();
            // We need to re-validate transactions
            let import = self
                .add_transactions_to_pool(chain, vec![tx.into()], TxOrigin::Local, &mut mem_pool)
                .pop()
                .expect("one result returned per added transaction; one added => one result; qed");

            match import {
                Ok(_) => {
                    ctrace!(OWN_TX, "Number of pending transactions: {:?}", mem_pool.num_pending_transactions());
                }
                Err(ref e) => {
                    ctrace!(OWN_TX, "Number of pending transactions: {:?}", mem_pool.num_pending_transactions());
                    cwarn!(OWN_TX, "Error importing transaction: {:?}", e);
                }
            }
            import
        };

        // ------------------------------------------------------------------
        // | NOTE Code below requires mem_pool and sealing_queue locks.     |
        // | Make sure to release the locks before calling that method.     |
        // ------------------------------------------------------------------
        if imported.is_ok() && self.options.reseal_on_own_transaction && self.transaction_reseal_allowed() && !self.engine_type().ignore_reseal_on_transaction()
            // Make sure to do it after transaction is imported and lock is dropped.
            // We need to create pending block and enable sealing.
            && self.engine.seals_internally()
        {
            // If new block has not been prepared (means we already had one)
            // or Engine might be able to seal internally,
            // we need to update sealing.
            self.update_sealing(chain, BlockId::Latest, false);
        }
        imported
    }

    fn ready_transactions(&self, size_limit: usize, range: Range<u64>) -> PendingVerifiedTransactions {
        self.mem_pool.read().top_transactions(size_limit, range)
    }

    fn count_pending_transactions(&self, range: Range<u64>) -> usize {
        self.mem_pool.read().count_pending_transactions(range)
    }

    fn start_sealing<C: MiningBlockChainClient + EngineInfo + TermInfo>(&self, client: &C) {
        cdebug!(MINER, "Start sealing");
        self.sealing_enabled.store(true, Ordering::Relaxed);
        // ------------------------------------------------------------------
        // | NOTE Code below requires mem_pool and sealing_queue locks.     |
        // | Make sure to release the locks before calling that method.     |
        // ------------------------------------------------------------------
        if self.transaction_reseal_allowed() {
            cdebug!(MINER, "Update sealing");
            self.update_sealing(client, BlockId::Latest, true);
        }
    }

    fn stop_sealing(&self) {
        cdebug!(MINER, "Stop sealing");
        self.sealing_enabled.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
pub mod test {
    use cio::IoService;
    use ckey::{Ed25519Private as Private, Signature};
    use coordinator::test_coordinator::TestCoordinator;
    use ctimer::TimerLoop;
    use ctypes::transaction::{Action, Transaction};

    use super::super::super::client::ClientConfig;
    use super::super::super::service::ClientIoMessage;
    use super::super::super::transaction::{UnverifiedTransaction, VerifiedTransaction};
    use super::*;
    use crate::client::Client;
    use crate::db::NUM_COLUMNS;

    #[test]
    fn check_add_transactions_result_idx() {
        let db = Arc::new(kvdb_memorydb::create(NUM_COLUMNS.unwrap()));
        let scheme = Scheme::new_test();
        let coordinator = Arc::new(TestCoordinator::default());
        let miner = Arc::new(Miner::with_scheme_for_test(&scheme, db.clone(), coordinator.clone()));

        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), 3, db.clone(), coordinator);
        let client = generate_test_client(db, Arc::clone(&miner), &scheme).unwrap();

        let private = Private::random();
        let transaction1: UnverifiedTransaction = VerifiedTransaction::new_with_sign(
            Transaction {
                seq: 30,
                fee: 40,
                network_id: "tc".into(),
                action: Action::Pay {
                    receiver: Public::random(),
                    quantity: 100,
                },
            },
            &private,
        )
        .into();

        // Invalid signature transaction which will be rejected before mem_pool.add
        let transaction2 = UnverifiedTransaction::new(
            Transaction {
                seq: 32,
                fee: 40,
                network_id: "tc".into(),
                action: Action::Pay {
                    receiver: Public::random(),
                    quantity: 100,
                },
            },
            Signature::random(),
            Public::random(),
        );

        let transactions = vec![transaction1.clone(), transaction2, transaction1];
        miner.add_transactions_to_pool(client.as_ref(), transactions, TxOrigin::Local, &mut mem_pool);
    }

    fn generate_test_client(db: Arc<dyn KeyValueDB>, miner: Arc<Miner>, scheme: &Scheme) -> Result<Arc<Client>, Error> {
        let timer_loop = TimerLoop::new(2);

        let client_config: ClientConfig = Default::default();
        let reseal_timer = timer_loop.new_timer_with_name("Client reseal timer");
        let io_service = IoService::<ClientIoMessage>::start("Client")?;

        let coordinator = Arc::new(TestCoordinator::default());
        Client::try_new(&client_config, scheme, db, miner, coordinator, io_service.channel(), reseal_timer)
    }
}
