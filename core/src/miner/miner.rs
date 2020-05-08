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
use super::mem_pool_types::{MemPoolInput, TxOrigin};
use super::{fetch_account_creator, MinerService, MinerStatus, TransactionImportResult};
use crate::account_provider::{AccountProvider, Error as AccountProviderError};
use crate::block::{ClosedBlock, IsBlock};
use crate::client::{
    AccountData, BlockChainTrait, BlockProducer, Client, EngineInfo, ImportBlock, MiningBlockChainClient, TermInfo,
};
use crate::consensus::{CodeChainEngine, EngineType};
use crate::error::Error;
use crate::scheme::Scheme;
use crate::transaction::{PendingVerifiedTransactions, UnverifiedTransaction, VerifiedTransaction};
use crate::types::{BlockId, TransactionId};
use ckey::{Ed25519Private as Private, Ed25519Public as Public, Password, PlatformAddress};
use cstate::{FindDoubleVoteHandler, TopLevelState, TopStateView};
use ctypes::errors::HistoryError;
use ctypes::transaction::{IncompleteTransaction, Transaction};
use ctypes::{BlockHash, TxHash};
use kvdb::KeyValueDB;
use parking_lot::{Mutex, RwLock};
use primitives::Bytes;
use rlp::Encodable;
use std::borrow::Borrow;
use std::collections::HashSet;
use std::convert::TryInto;
use std::iter::FromIterator;
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
    pub author: Public,
    pub extra_data: Bytes,
}

type TransactionListener = Box<dyn Fn(&[TxHash]) + Send + Sync>;

pub struct Miner {
    mem_pool: Arc<RwLock<MemPool>>,
    transaction_listener: RwLock<Vec<TransactionListener>>,
    next_allowed_reseal: Mutex<Instant>,
    params: RwLock<AuthoringParams>,
    engine: Arc<dyn CodeChainEngine>,
    options: MinerOptions,

    sealing_enabled: AtomicBool,

    accounts: Arc<AccountProvider>,
    malicious_users: RwLock<HashSet<Public>>,
    immune_users: RwLock<HashSet<Public>>,
}

impl Miner {
    pub fn new(
        options: MinerOptions,
        scheme: &Scheme,
        accounts: Arc<AccountProvider>,
        db: Arc<dyn KeyValueDB>,
    ) -> Arc<Self> {
        Arc::new(Self::new_raw(options, scheme, accounts, db))
    }

    pub fn with_scheme_for_test(scheme: &Scheme, db: Arc<dyn KeyValueDB>) -> Self {
        Self::new_raw(Default::default(), scheme, AccountProvider::transient_provider(), db)
    }

    fn new_raw(
        options: MinerOptions,
        scheme: &Scheme,
        accounts: Arc<AccountProvider>,
        db: Arc<dyn KeyValueDB>,
    ) -> Self {
        let mem_limit = options.mem_pool_memory_limit.unwrap_or_else(usize::max_value);
        let mem_pool = Arc::new(RwLock::new(MemPool::with_limits(
            options.mem_pool_size,
            mem_limit,
            options.mem_pool_fee_bump_shift,
            db,
        )));

        Self {
            mem_pool,
            transaction_listener: RwLock::new(vec![]),
            next_allowed_reseal: Mutex::new(Instant::now()),
            params: RwLock::new(AuthoringParams::default()),
            engine: scheme.engine.clone(),
            options,
            sealing_enabled: AtomicBool::new(true),
            accounts,
            malicious_users: RwLock::new(HashSet::new()),
            immune_users: RwLock::new(HashSet::new()),
        }
    }

    pub fn recover_from_db(&self, client: &Client) {
        self.mem_pool.write().recover_from_db(client);
    }

    /// Set a callback to be notified about imported transactions' hashes.
    pub fn add_transactions_listener(&self, f: Box<dyn Fn(&[TxHash]) + Send + Sync>) {
        self.transaction_listener.write().push(f);
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
                if default_origin.is_local() {
                    self.immune_users.write().insert(signer_public);
                }

                let origin = if self.accounts.has_account(&signer_public).unwrap_or_default() {
                    TxOrigin::Local
                } else {
                    default_origin
                };

                if self.malicious_users.read().contains(&signer_public) {
                    // FIXME: just to skip, think about another way.
                    return Ok(())
                }
                if client.transaction_block(&TransactionId::Hash(hash)).is_some() {
                    cdebug!(MINER, "Rejected transaction {:?}: already in the blockchain", hash);
                    return Err(HistoryError::TransactionAlreadyImported.into())
                }
                let immune_users = self.immune_users.read();
                let tx: VerifiedTransaction = tx
                    .verify_basic()
                    .map_err(From::from)
                    .and_then(|_| {
                        let common_params = client.common_params(best_header.hash().into()).unwrap();
                        self.engine.verify_transaction_with_params(&tx, &common_params)
                    })
                    .and_then(|_| Ok(tx.try_into()?))
                    .map_err(|e| {
                        match e {
                            Error::Syntax(_) if !origin.is_local() && !immune_users.contains(&signer_public) => {
                                self.malicious_users.write().insert(signer_public);
                            }
                            _ => {}
                        }
                        cdebug!(MINER, "Rejected transaction {:?} with error {:?}", hash, e);
                        e
                    })?;

                let tx_hash = tx.hash();

                to_insert.push(MemPoolInput::new(tx, origin));
                tx_hashes.push(tx_hash);
                Ok(())
            })
            .collect();

        let fetch_account = fetch_account_creator(client);

        let insertion_results = mem_pool.add(to_insert, current_block_number, current_timestamp, &fetch_account);

        debug_assert_eq!(insertion_results.len(), intermediate_results.iter().filter(|r| r.is_ok()).count());
        let mut insertion_results_index = 0;
        let results = intermediate_results
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
    fn prepare_and_seal_block<
        C: AccountData + BlockChainTrait + BlockProducer + EngineInfo + FindDoubleVoteHandler + TermInfo,
    >(
        &self,
        parent_block_id: BlockId,
        chain: &C,
    ) -> Result<Option<ClosedBlock>, Error> {
        let (transactions, mut open_block, block_number, block_tx_signer, block_tx_seq) = {
            ctrace!(MINER, "prepare_block: No existing work - making new block");
            let params = self.params.read().clone();
            let open_block = chain.prepare_open_block(parent_block_id, params.author, params.extra_data);
            let (block_number, parent_hash) = {
                let header = open_block.block().header();
                let block_number = header.number();
                let parent_hash = *header.parent_hash();
                (block_number, parent_hash)
            };
            let max_body_size = chain.common_params(parent_hash.into()).unwrap().max_body_size();
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
            if self.malicious_users.read().contains(&signer_public) {
                invalid_transactions.push(tx.hash());
                continue
            }
            if invalid_tx_users.contains(&signer_public) {
                // The previous transaction has failed
                continue
            }

            let hash = tx.hash();
            let start = Instant::now();
            // Check whether transaction type is allowed for sender
            let result = open_block.push_transaction(tx, chain, parent_header.number(), parent_header.timestamp());

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
            for action in actions {
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
                open_block.push_transaction(tx, chain, parent_header.number(), parent_header.timestamp())?;
            }
        }
        let block = open_block.close()?;

        let fetch_seq = |p: &Public| chain.latest_seq(p);

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
        self.sealing_enabled.load(Ordering::Relaxed) && (Instant::now() > *self.next_allowed_reseal.lock())
    }
}

impl MinerService for Miner {
    type State = TopLevelState;

    fn status(&self) -> MinerStatus {
        let status = self.mem_pool.read().status();
        MinerStatus {
            transactions_in_pending_queue: status.pending,
            transactions_in_future_queue: status.future,
        }
    }

    fn authoring_params(&self) -> AuthoringParams {
        self.params.read().clone()
    }

    fn set_author(&self, pubkey: Public) -> Result<(), AccountProviderError> {
        self.params.write().author = pubkey;

        if self.engine_type().need_signer_key() {
            ctrace!(MINER, "Set author to {:?}", pubkey);
            // Sign test message
            self.accounts.get_unlocked_account(&pubkey)?.sign(&Default::default())?;
            self.engine.set_signer(Arc::clone(&self.accounts), pubkey);
        }
        Ok(())
    }

    fn get_author(&self) -> Public {
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

    fn chain_new_blocks<C>(&self, chain: &C, _imported: &[BlockHash], _invalid: &[BlockHash], _enacted: &[BlockHash])
    where
        C: AccountData + BlockChainTrait + BlockProducer + EngineInfo + ImportBlock, {
        ctrace!(MINER, "chain_new_blocks");

        {
            let fetch_account = fetch_account_creator(chain);
            let current_block_number = chain.chain_info().best_block_number;
            let current_timestamp = chain.chain_info().best_block_timestamp;
            let mut mem_pool = self.mem_pool.write();
            mem_pool.remove_old(&fetch_account, current_block_number, current_timestamp);
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
        C: AccountData + BlockChainTrait + BlockProducer + EngineInfo + ImportBlock + FindDoubleVoteHandler + TermInfo,
    {
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

    fn import_incomplete_transaction<C: MiningBlockChainClient + AccountData + EngineInfo + TermInfo>(
        &self,
        client: &C,
        account_provider: &AccountProvider,
        tx: IncompleteTransaction,
        platform_address: PlatformAddress,
        passphrase: Option<Password>,
        seq: Option<u64>,
    ) -> Result<(TxHash, u64), Error> {
        let pubkey = platform_address.try_into_pubkey()?;
        let seq = match seq {
            Some(seq) => seq,
            None => get_next_seq(self.future_transactions(), &[pubkey])
                .map(|seq| {
                    cwarn!(RPC, "There are future transactions for {}", platform_address);
                    seq
                })
                .unwrap_or_else(|| {
                    let size_limit = client
                        .common_params(BlockId::Latest)
                        .expect("Common params of the latest block always exists")
                        .max_body_size();
                    const DEFAULT_RANGE: Range<u64> = 0..u64::MAX;
                    get_next_seq(self.ready_transactions(size_limit, DEFAULT_RANGE).transactions, &[pubkey])
                        .map(|seq| {
                            cdebug!(RPC, "There are ready transactions for {}", platform_address);
                            seq
                        })
                        .unwrap_or_else(|| client.latest_seq(&pubkey))
                }),
        };
        let tx = tx.complete(seq);
        let tx_hash = tx.hash();
        let account = account_provider.get_account(&pubkey, passphrase.as_ref())?;
        let sig = account.sign(&tx_hash)?;
        let signer_public = account.public()?;
        let unverified = UnverifiedTransaction::new(tx, sig, signer_public);
        let signed: VerifiedTransaction = unverified.try_into()?;
        let hash = signed.hash();
        self.import_own_transaction(client, signed)?;

        Ok((hash, seq))
    }

    fn ready_transactions(&self, size_limit: usize, range: Range<u64>) -> PendingVerifiedTransactions {
        self.mem_pool.read().top_transactions(size_limit, range)
    }

    fn count_pending_transactions(&self, range: Range<u64>) -> usize {
        self.mem_pool.read().count_pending_transactions(range)
    }

    fn future_included_count_pending_transactions(&self, range: Range<u64>) -> usize {
        self.mem_pool.read().future_included_count_pending_transactions(range)
    }

    fn future_pending_transactions(&self, range: Range<u64>) -> PendingVerifiedTransactions {
        self.mem_pool.read().get_future_pending_transactions(usize::max_value(), range)
    }

    /// Get a list of all future transactions.
    fn future_transactions(&self) -> Vec<VerifiedTransaction> {
        self.mem_pool.read().future_transactions()
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

    fn get_malicious_users(&self) -> Vec<Public> {
        Vec::from_iter(self.malicious_users.read().iter().map(Clone::clone))
    }

    fn release_malicious_users(&self, prisoners: Vec<Public>) {
        let mut malicious_users = self.malicious_users.write();
        for prisoner in &prisoners {
            malicious_users.remove(prisoner);
        }
    }

    fn imprison_malicious_users(&self, prisoners: Vec<Public>) {
        let mut malicious_users = self.malicious_users.write();
        for prisoner in prisoners {
            malicious_users.insert(prisoner);
        }
    }

    fn get_immune_users(&self) -> Vec<Public> {
        Vec::from_iter(self.immune_users.read().iter().map(Clone::clone))
    }

    fn register_immune_users(&self, immune_users: Vec<Public>) {
        let mut immune_users_lock = self.immune_users.write();
        for user in immune_users {
            immune_users_lock.insert(user);
        }
    }
}

fn get_next_seq(transactions: impl IntoIterator<Item = VerifiedTransaction>, pubkeys: &[Public]) -> Option<u64> {
    let mut txes =
        transactions.into_iter().filter(|tx| pubkeys.contains(&tx.signer_public())).map(|tx| tx.transaction().seq);
    if let Some(first) = txes.next() {
        Some(txes.fold(first, std::cmp::max) + 1)
    } else {
        None
    }
}

#[cfg(test)]
pub mod test {
    use cio::IoService;
    use ckey::{Ed25519Private as Private, Signature};
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
        let miner = Arc::new(Miner::with_scheme_for_test(&scheme, db.clone()));

        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), 3, db.clone());
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

        Client::try_new(&client_config, scheme, db, miner, io_service.channel(), reseal_timer)
    }
}
