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

use super::mem_pool::{Error as MemPoolError, MemPool};
use super::{MinerService, MinerStatus};
use crate::account_provider::{AccountProvider, Error as AccountProviderError};
use crate::block::{ClosedBlock, IsBlock};
use crate::client::{BlockChainClient, BlockChainTrait, BlockProducer, EngineInfo, ImportBlock, TermInfo};
use crate::consensus::{ConsensusEngine, EngineType};
use crate::error::Error;
use crate::scheme::Scheme;
use crate::transaction::PendingTransactions;
use crate::types::{BlockId, TransactionId};
use ckey::Address;
use coordinator::validator::{Transaction, TxOrigin, Validator};
use cstate::TopLevelState;
use ctypes::errors::HistoryError;
use ctypes::{BlockHash, TxHash};
use kvdb::KeyValueDB;
use parking_lot::{Mutex, RwLock};
use primitives::Bytes;
use std::ops::Range;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

/// Configures the behaviour of the miner.
#[derive(Debug, PartialEq)]
pub struct MinerOptions {
    /// Reseal on receipt of new external transactions.
    pub reseal_on_external_transaction: bool,
    /// Reseal on receipt of new local transactions.
    pub reseal_on_own_transaction: bool,
    /// Minimum period between transaction-inspired reseals.
    pub reseal_min_period: Duration,
    /// Disable the reseal timer
    pub no_reseal_timer: bool,
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
            no_reseal_timer: false,
            mem_pool_size: 8192,
            mem_pool_memory_limit: Some(2 * 1024 * 1024),
            mem_pool_fee_bump_shift: 3,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct AuthoringParams {
    pub author: Address,
    pub extra_data: Bytes,
}

type TransactionListener = Box<dyn Fn(&[TxHash]) + Send + Sync>;

pub struct Miner {
    mem_pool: Arc<RwLock<MemPool>>,
    transaction_listener: RwLock<Vec<TransactionListener>>,
    next_allowed_reseal: Mutex<Instant>,
    params: RwLock<AuthoringParams>,
    engine: Arc<dyn ConsensusEngine>,
    options: MinerOptions,
    sealing_enabled: AtomicBool,
    validator: Arc<dyn Validator>,
}

impl Miner {
    pub fn new(
        options: MinerOptions,
        scheme: &Scheme,
        db: Arc<dyn KeyValueDB>,
        validator: Arc<dyn Validator>,
    ) -> Arc<Self> {
        Arc::new(Self::new_raw(options, scheme, db, validator))
    }

    pub fn with_scheme_for_test(scheme: &Scheme, db: Arc<dyn KeyValueDB>, validator: Arc<dyn Validator>) -> Self {
        Self::new_raw(Default::default(), scheme, db, validator)
    }

    fn new_raw(options: MinerOptions, scheme: &Scheme, db: Arc<dyn KeyValueDB>, validator: Arc<dyn Validator>) -> Self {
        let mem_limit = options.mem_pool_memory_limit.unwrap_or_else(usize::max_value);
        let mem_pool =
            Arc::new(RwLock::new(MemPool::with_limits(options.mem_pool_size, mem_limit, db, validator.clone())));

        Self {
            mem_pool,
            transaction_listener: RwLock::new(vec![]),
            next_allowed_reseal: Mutex::new(Instant::now()),
            params: RwLock::new(AuthoringParams::default()),
            engine: scheme.engine.clone(),
            options,
            sealing_enabled: AtomicBool::new(true),
            validator,
        }
    }

    pub fn recover_from_db(&self) {
        self.mem_pool.write().recover_from_db();
    }

    /// Set a callback to be notified about imported transactions' hashes.
    pub fn add_transactions_listener(&self, f: Box<dyn Fn(&[TxHash]) + Send + Sync>) {
        self.transaction_listener.write().push(f);
    }

    pub fn get_options(&self) -> &MinerOptions {
        &self.options
    }

    fn add_transactions_to_pool<C: BlockChainTrait + EngineInfo>(
        &self,
        client: &C,
        transactions: Vec<Transaction>,
        origin: TxOrigin,
        mem_pool: &mut MemPool,
    ) -> Vec<Result<(), Error>> {
        let current_block_number = client.chain_info().best_block_number;
        let current_timestamp = client.chain_info().best_block_timestamp;
        let mut inserted = Vec::with_capacity(transactions.len());
        let mut to_insert = Vec::new();
        let mut tx_hashes = Vec::new();

        let intermediate_results: Vec<Result<(), Error>> = transactions
            .into_iter()
            .map(|tx| {
                let hash = tx.hash();
                if client.transaction_block(&TransactionId::Hash(hash)).is_some() {
                    cdebug!(MINER, "Rejected transaction {:?}: already in the blockchain", hash);
                    Err(HistoryError::TransactionAlreadyImported.into())
                } else {
                    to_insert.push(tx);
                    tx_hashes.push(hash);
                    Ok(())
                }
            })
            .collect();

        let insertion_results = mem_pool.add(to_insert, origin, current_block_number, current_timestamp);

        debug_assert_eq!(insertion_results.len(), intermediate_results.iter().filter(|r| r.is_ok()).count());
        let mut insertion_results_index = 0;
        let results = intermediate_results
            .into_iter()
            .map(|res| match res {
                Err(e) => Err(e),
                Ok(()) => {
                    let idx = insertion_results_index;
                    insertion_results[idx].clone().map_err(MemPoolError::into_core_error)?;
                    inserted.push(tx_hashes[idx]);
                    insertion_results_index += 1;
                    Ok(())
                }
            })
            .collect();

        for listener in &*self.transaction_listener.read() {
            listener(&inserted);
        }

        results
    }

    pub fn delete_all_pending_transactions(&self) {
        let mut mem_pool = self.mem_pool.write();
        mem_pool.remove_all();
    }

    /// Prepares new block for sealing including top transactions from queue and seal it.
    fn prepare_and_seal_block<C: BlockChainTrait + BlockProducer + EngineInfo + TermInfo>(
        &self,
        parent_block_id: BlockId,
        chain: &C,
    ) -> Result<Option<ClosedBlock>, Error> {
        let (transactions, mut open_block) = {
            ctrace!(MINER, "prepare_block: No existing work - making new block");
            let params = self.params.read().clone();
            let open_block = chain.prepare_open_block(parent_block_id, params.author, params.extra_data);
            let header = open_block.block().header();
            let parent_hash = *header.parent_hash();
            let max_body_size = chain.consensus_params(parent_hash.into()).unwrap().max_body_size();
            const DEFAULT_RANGE: Range<u64> = 0..::std::u64::MAX;

            // NOTE: This lock should be acquired after `prepare_open_block` to prevent deadlock
            let mem_pool = self.mem_pool.read();
            // TODO: Create a gas_limit parameter and use it
            let transactions = mem_pool.top_transactions(max_body_size, max_body_size, DEFAULT_RANGE).transactions;

            (transactions, open_block)
        };

        let parent_header = {
            let parent_hash = open_block.header().parent_hash();
            chain.block_header(&BlockId::Hash(*parent_hash)).expect("Parent header MUST exist")
        };

        assert!(self.engine.seals_internally(), "If a signer is not prepared, prepare_block should not be called");
        let seal = self.engine.generate_seal(None, &parent_header.decode());
        if let Some(seal_bytes) = seal.seal_fields() {
            open_block.seal(seal_bytes).expect("Sealing always success");
        } else {
            return Ok(None)
        }
        self.engine.on_open_block(open_block.inner_mut())?;

        let evidences = self.engine.fetch_evidences();

        let validator = &*self.validator;

        open_block.open(validator, evidences);
        open_block.execute_transactions(validator, transactions);
        let closed_block = open_block.close(validator)?;
        Ok(Some(closed_block))
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
        self.sealing_enabled.load(Ordering::Relaxed) && (Instant::now() > *self.next_allowed_reseal.lock())
    }
}

impl MinerService for Miner {
    type State = TopLevelState;

    fn status(&self) -> MinerStatus {
        let status = self.mem_pool.read().status();
        MinerStatus {
            transactions_in_pending_queue: status.pending,
        }
    }

    fn authoring_params(&self) -> AuthoringParams {
        self.params.read().clone()
    }

    fn set_author(&self, ap: Arc<AccountProvider>, address: Address) -> Result<(), AccountProviderError> {
        self.params.write().author = address;

        if self.engine_type().need_signer_key() {
            ap.get_unlocked_account(&address)?.sign(&Default::default())?;
            self.engine.set_signer(ap, address);
            Ok(())
        } else {
            Ok(())
        }
    }

    fn get_author_address(&self) -> Address {
        self.params.read().author
    }

    fn set_extra_data(&self, extra_data: Bytes) {
        self.params.write().extra_data = extra_data;
    }

    fn transactions_limit(&self) -> usize {
        self.mem_pool.read().limit()
    }

    fn set_transactions_limit(&self, limit: usize) {
        self.mem_pool.write().set_limit(limit)
    }

    fn chain_new_blocks<C>(&self, chain: &C, _imported: &[BlockHash], _invalid: &[BlockHash], enacted: &[BlockHash])
    where
        C: BlockChainTrait + BlockProducer + EngineInfo + ImportBlock, {
        ctrace!(MINER, "chain_new_blocks");

        {
            let mut mem_pool = self.mem_pool.write();
            let to_remove: Vec<_> = enacted
                .iter()
                .flat_map(|hash| chain.block(&BlockId::from(*hash)))
                .flat_map(|block| block.view().transactions())
                .map(|tx| tx.hash())
                .collect();
            mem_pool.remove(&to_remove);
            mem_pool.remove_old();
        }

        if !self.options.no_reseal_timer {
            chain.set_min_timer();
        }
    }

    fn engine_type(&self) -> EngineType {
        self.engine.engine_type()
    }

    fn update_sealing<C>(&self, chain: &C, parent_block: BlockId, allow_empty_block: bool)
    where
        C: BlockChainTrait + BlockProducer + EngineInfo + ImportBlock + TermInfo, {
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
        *self.next_allowed_reseal.lock() = Instant::now() + self.options.reseal_min_period;
        if !self.options.no_reseal_timer {
            chain.set_min_timer();
        }
    }

    fn import_external_transactions<C: BlockChainClient + BlockProducer + EngineInfo + TermInfo>(
        &self,
        client: &C,
        transactions: Vec<Transaction>,
    ) -> Vec<Result<(), Error>> {
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

    fn import_own_transaction<C: BlockChainTrait + BlockProducer + ImportBlock + EngineInfo + TermInfo>(
        &self,
        chain: &C,
        tx: Transaction,
    ) -> Result<(), Error> {
        ctrace!(OWN_TX, "Importing transaction: {:?}", tx);

        let imported = {
            // Be sure to release the lock before we call prepare_work_sealing
            let mut mem_pool = self.mem_pool.write();
            // We need to re-validate transactions
            let import = self
                .add_transactions_to_pool(chain, vec![tx], TxOrigin::Local, &mut mem_pool)
                .pop()
                .expect("one result returned per added transaction; one added => one result; qed");

            match import {
                Ok(_) => {
                    ctrace!(OWN_TX, "Status: {:?}", mem_pool.status());
                }
                Err(ref e) => {
                    ctrace!(OWN_TX, "Status: {:?}", mem_pool.status());
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

    fn ready_transactions(&self, gas_limit: usize, size_limit: usize, range: Range<u64>) -> PendingTransactions {
        // TODO: Create a gas_limit parameter and use it.
        self.mem_pool.read().top_transactions(gas_limit, size_limit, range)
    }

    fn count_pending_transactions(&self, range: Range<u64>) -> usize {
        self.mem_pool.read().count_pending_transactions(range)
    }

    fn start_sealing<C: BlockChainClient + BlockProducer + EngineInfo + TermInfo>(&self, client: &C) {
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
    #[test]
    fn check_add_transactions_result_idx() {
        todo!()
        //TODO: Write test after implementing a mockup coordinator
    }

    fn generate_test_client(db: Arc<dyn KeyValueDB>, miner: Arc<Miner>, scheme: &Scheme) -> Result<Arc<Client>, Error> {
        todo!()
        //TODO: Write test after implementing a mockup coordinator
    }
}
