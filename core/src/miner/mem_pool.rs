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

use super::backup;
use super::mem_pool_types::{MemPoolStatus, PoolingInstant, TransactionPool};
use crate::client::{AccountData, BlockChainTrait};
use crate::Error as CoreError;
use coordinator::{
    context::DummyContext, validator::Transaction, validator::TransactionWithMetadata, validator::TxOrigin,
    validator::Validator, Coordinator,
};
use ctypes::errors::{HistoryError, RuntimeError, SyntaxError};
use ctypes::{BlockNumber, TxHash};
use kvdb::{DBTransaction, KeyValueDB};
use std::collections::HashMap;
use std::ops::{Deref, Range};
use std::sync::Arc;
const DEFAULT_POOLING_PERIOD: BlockNumber = 128;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    History(HistoryError),
    Runtime(RuntimeError),
    Syntax(SyntaxError),
}

impl Error {
    pub fn into_core_error(self) -> CoreError {
        match self {
            Error::History(err) => CoreError::History(err),
            Error::Runtime(err) => CoreError::Runtime(err),
            Error::Syntax(err) => CoreError::Syntax(err),
        }
    }
}

impl From<HistoryError> for Error {
    fn from(err: HistoryError) -> Error {
        Error::History(err)
    }
}

impl From<RuntimeError> for Error {
    fn from(err: RuntimeError) -> Error {
        Error::Runtime(err)
    }
}

impl From<SyntaxError> for Error {
    fn from(err: SyntaxError) -> Error {
        Error::Syntax(err)
    }
}

pub struct MemPool {
    coordinator: Coordinator<DummyContext>,
    /// list of all transactions in the pool
    transaction_pool: TransactionPool,
    /// The count(number) limit of each queue
    queue_count_limit: usize,
    /// The memory limit of each queue
    queue_memory_limit: usize,
    /// Next id that should be assigned to a transaction imported to the pool
    next_transaction_id: u64,
    /// Arc of KeyValueDB in which the backup information is stored.
    db: Arc<dyn KeyValueDB>,
}

impl MemPool {
    /// Create new instance of this Queue with specified limits
    pub fn with_limits(limit: usize, memory_limit: usize, db: Arc<dyn KeyValueDB>) -> Self {
        MemPool {
            coordinator: Coordinator::<DummyContext>::default(),
            transaction_pool: TransactionPool::new(),
            queue_count_limit: limit,
            queue_memory_limit: memory_limit,
            next_transaction_id: 0,
            db,
        }
    }

    /// Set the new limit for `current` and `future` queue.
    pub fn set_limit(&mut self, limit: usize) {
        self.queue_count_limit = limit;
    }

    /// Enforce the limit to the current/future queue
    fn enforce_limit(&mut self, batch: &mut DBTransaction) {
        // Get transaction orders to drop from each queue (current/future)
        fn get_txs_to_drop(
            txs: &HashMap<TxHash, TransactionWithMetadata>,
            limit: usize,
            memory_limit: usize,
        ) -> Vec<TransactionWithMetadata> {
            let mut count = 0;
            let mut mem_usage = 0;
            txs.values()
                .filter(|tx| {
                    count += 1;
                    mem_usage += tx.size();
                    mem_usage > memory_limit || count > limit
                })
                .cloned()
                .collect()
        }

        let to_drop = if self.transaction_pool.mem_usage > self.queue_memory_limit
            || self.transaction_pool.count > self.queue_count_limit
        {
            get_txs_to_drop(&self.transaction_pool.pool, self.queue_count_limit, self.queue_memory_limit)
        } else {
            vec![]
        };

        for tx in to_drop {
            let hash = &tx.hash();
            backup::remove_item(batch, hash.deref());
            self.transaction_pool.remove(hash);
        }
    }

    /// Returns current limit of transactions in the pool.
    pub fn limit(&self) -> usize {
        self.queue_count_limit
    }

    /// Returns current status for this pool
    pub fn status(&self) -> MemPoolStatus {
        MemPoolStatus {
            pending: self.transaction_pool.len(),
        }
    }

    /// Returns current status for this pool
    pub fn pending_transactions(&self) -> usize {
        self.transaction_pool.len()
    }

    /// Add signed transaction to pool to be verified and imported.
    ///
    /// NOTE details_provider methods should be cheap to compute
    /// otherwise it might open up an attack vector.
    pub fn add(
        &mut self,
        transactions: Vec<Transaction>,
        origin: TxOrigin,
        inserted_block_number: PoolingInstant,
        inserted_timestamp: u64,
    ) -> Vec<bool> {
        let mut batch = backup::backup_batch_with_capacity(transactions.len());
        let mut results = Vec::with_capacity(transactions.len());
        for tx in transactions {
            let id = self.next_transaction_id;
            self.next_transaction_id += 1;

            let hash = &tx.hash();
            if self.coordinator.check_transaction(&tx) {
                let tx = TransactionWithMetadata::new(tx, origin, inserted_block_number, inserted_timestamp, id);
                if self.transaction_pool.contains(hash) {
                    // This transaction is already in the pool.
                    results.push(false);
                } else {
                    self.transaction_pool.insert(tx);
                    results.push(true);
                }
            } else {
                // This transaction is invalid.
                results.push(false);
            }
        }
        self.enforce_limit(&mut batch);

        self.db.write(batch).expect("Low level database error. Some issue with disk?");

        assert_eq!(results.len(), transactions.len());
        results
    }

    pub fn remove_all(&mut self) {
        self.transaction_pool.clear();
    }

    // Recover MemPool state from db stored data
    pub fn recover_from_db<C: AccountData + BlockChainTrait>(&mut self, client: &C) {
        let by_hash = backup::recover_to_data(self.db.as_ref());

        let mut max_insertion_id = 0u64;

        for (hash, tx) in by_hash.iter() {
            assert_eq!(*hash, *tx.hash());
            if tx.insertion_id > max_insertion_id {
                max_insertion_id = tx.insertion_id;
            }
            self.transaction_pool.insert(tx.clone());
        }

        self.next_transaction_id = max_insertion_id + 1;
    }

    /// Removes all elements (in any state) from the pool
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.transaction_pool.clear();
    }

    pub fn top_transactions(&self, gas_limit: usize, size_limit: usize, range: Range<u64>) -> Vec<Transaction> {
        let mut current_gas: usize = 0;
        let mut current_size: usize = 0;
        let unordered_transactions: Vec<_> =
            self.transaction_pool.pool.values().filter(|tx| range.contains(&tx.inserted_timestamp)).collect();
        let ordered_transactions = self.coordinator.fetch_transactions_for_block(&unordered_transactions);
        let chosen_transactions = ordered_transactions
            .iter()
            .take_while(|tx_with_gas| {
                let size = tx_with_gas.size();
                let gas = tx_with_gas.gas;
                current_size += size;
                current_gas += gas;
                current_gas < gas_limit && current_size < size_limit
            })
            .map(|tx_with_gas| tx_with_gas.tx)
            .collect();
        chosen_transactions
    }

    pub fn remove_old(&mut self) {
        let transactions: Vec<_> = self.transaction_pool.pool.values().collect();
        let transactions_after_filtering = self.coordinator.remove_old_transactions(&transactions);
        // TODO: optimization might be required
        self.transaction_pool.clear();
        for tx in transactions_after_filtering {
            self.transaction_pool.insert(tx);
        }
    }

    // Used to remove transactions from mempool after accepting blocks
    pub fn remove(&mut self, transaction_hashes: &[TxHash]) {
        for hash in transaction_hashes {
            self.transaction_pool.remove(hash);
        }
    }

    pub fn count_pending_transactions(&self, range: Range<u64>) -> usize {
       self.transaction_pool.pool.values().filter(|tx| {
           range.contains(&tx.inserted_timestamp)
       })
       .count()
    }
}

#[cfg(test)]
pub mod test {
    use std::cmp::Ordering;

    use crate::client::{AccountData, TestBlockChainClient};
    use ckey::{Generator, KeyPair, Random};
    use ctypes::transaction::{Action, Transaction};

    use super::*;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn origin_ordering() {
        assert_eq!(TxOrigin::Local.cmp(&TxOrigin::External), Ordering::Less);
        assert_eq!(TxOrigin::External.cmp(&TxOrigin::Local), Ordering::Greater);
    }

    #[test]
    fn pay_transaction_increases_cost() {
        let fee = 100;
        let quantity = 100_000;
        let receiver = 1u64.into();
        let keypair = Random.generate().unwrap();
        let tx = Transaction {
            seq: 0,
            fee,
            network_id: "tc".into(),
            action: Action::Pay {
                receiver,
                quantity,
            },
        };
        let signed = SignedTransaction::new_with_sign(tx, keypair.private());
        let item = MemPoolItem::new(signed, TxOrigin::Local, 0, 0, 0);

        assert_eq!(fee + quantity, item.cost());
    }

    #[test]
    fn txorigin_encode_and_decode() {
        rlp_encode_and_decode_test!(TxOrigin::External);
    }

    #[test]
    fn signed_transaction_encode_and_decode() {
        let receiver = 0u64.into();
        let keypair = Random.generate().unwrap();
        let tx = Transaction {
            seq: 0,
            fee: 100,
            network_id: "tc".into(),
            action: Action::Pay {
                receiver,
                quantity: 100_000,
            },
        };
        let signed = SignedTransaction::new_with_sign(tx, keypair.private());

        rlp_encode_and_decode_test!(signed);
    }

    #[test]
    fn mempool_item_encode_and_decode() {
        let keypair = Random.generate().unwrap();
        let tx = Transaction {
            seq: 0,
            fee: 10,
            network_id: "tc".into(),
            action: Action::Pay {
                receiver: Default::default(),
                quantity: 0,
            },
        };
        let signed = SignedTransaction::new_with_sign(tx, keypair.private());
        let item = MemPoolItem::new(signed, TxOrigin::Local, 0, 0, 0);

        rlp_encode_and_decode_test!(item);
    }

    #[test]
    fn db_backup_and_recover() {
        //setup test_client
        let test_client = TestBlockChainClient::new();
        let keypair = Random.generate().unwrap();
        let default_addr = public_to_address(keypair.public());
        test_client.set_seq(default_addr, 4u64);
        test_client.set_balance(default_addr, u64::max_value());

        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), 3, db.clone(), Default::default());

        let fetch_account = fetch_account_creator(&test_client);

        let inserted_block_number = 1;
        let inserted_timestamp = 100;
        let mut inputs: Vec<MemPoolInput> = Vec::new();

        inputs.push(create_mempool_input_with_pay(1u64, keypair));
        inputs.push(create_mempool_input_with_pay(3u64, keypair));
        inputs.push(create_mempool_input_with_pay(5u64, keypair));
        mem_pool.add(inputs, inserted_block_number, inserted_timestamp, &fetch_account);

        let inserted_block_number = 11;
        let inserted_timestamp = 200;
        let mut inputs: Vec<MemPoolInput> = Vec::new();
        inputs.push(create_mempool_input_with_pay(2u64, keypair));
        inputs.push(create_mempool_input_with_pay(4u64, keypair));
        mem_pool.add(inputs, inserted_block_number, inserted_timestamp, &fetch_account);

        let inserted_block_number = 20;
        let inserted_timestamp = 300;
        let mut inputs: Vec<MemPoolInput> = Vec::new();
        inputs.push(create_mempool_input_with_pay(6u64, keypair));
        inputs.push(create_mempool_input_with_pay(8u64, keypair));
        inputs.push(create_mempool_input_with_pay(10u64, keypair));
        mem_pool.add(inputs, inserted_block_number, inserted_timestamp, &fetch_account);

        let inserted_block_number = 21;
        let inserted_timestamp = 400;
        let mut inputs: Vec<MemPoolInput> = Vec::new();
        inputs.push(create_mempool_input_with_pay(7u64, keypair));
        mem_pool.add(inputs, inserted_block_number, inserted_timestamp, &fetch_account);

        let mut mem_pool_recovered = MemPool::with_limits(8192, usize::max_value(), 3, db, Default::default());
        mem_pool_recovered.recover_from_db(&test_client);

        assert_eq!(mem_pool_recovered.first_seqs, mem_pool.first_seqs);
        assert_eq!(mem_pool_recovered.next_seqs, mem_pool.next_seqs);
        assert_eq!(mem_pool_recovered.by_signer_public, mem_pool.by_signer_public);
        assert_eq!(mem_pool_recovered.is_local_account, mem_pool.is_local_account);
        assert_eq!(mem_pool_recovered.next_transaction_id, mem_pool.next_transaction_id);
        assert_eq!(mem_pool_recovered.by_hash, mem_pool.by_hash);
        assert_eq!(mem_pool_recovered.queue_count_limit, mem_pool.queue_count_limit);
        assert_eq!(mem_pool_recovered.queue_memory_limit, mem_pool.queue_memory_limit);
        assert_eq!(mem_pool_recovered.transaction_queue, mem_pool.transaction_queue);
        assert_eq!(mem_pool_recovered.future, mem_pool.future);
    }

    fn create_signed_pay(seq: u64, keypair: KeyPair) -> SignedTransaction {
        let receiver = 1u64.into();
        let tx = Transaction {
            seq,
            fee: 100,
            network_id: "tc".into(),
            action: Action::Pay {
                receiver,
                quantity: 100_000,
            },
        };
        SignedTransaction::new_with_sign(tx, keypair.private())
    }

    fn create_signed_pay_with_fee(seq: u64, fee: u64, keypair: KeyPair) -> SignedTransaction {
        let receiver = 1u64.into();
        let tx = Transaction {
            seq,
            fee,
            network_id: "tc".into(),
            action: Action::Pay {
                receiver,
                quantity: 100_000,
            },
        };
        SignedTransaction::new_with_sign(tx, keypair.private())
    }

    fn create_mempool_input_with_pay(seq: u64, keypair: KeyPair) -> MemPoolInput {
        let signed = create_signed_pay(seq, keypair);
        MemPoolInput::new(signed, TxOrigin::Local)
    }

    fn abbreviated_mempool_add(
        test_client: &TestBlockChainClient,
        mem_pool: &mut MemPool,
        txs: Vec<Transaction>,
        check_type: TransactionCheckType,
    ) -> Vec<bool> {
        let inserted_block_number = 1;
        let inserted_timestamp = 100;
        mem_pool.add(txs, check_type, inserted_block_number, inserted_timestamp)
    }

    #[test]
    fn local_transactions_whose_fees_are_under_the_mem_pool_min_fee_should_not_be_rejected() {
        let test_client = TestBlockChainClient::new();

        // Set the pay transaction minimum fee
        let fees =
            MemPoolMinFees::create_from_options(Some(150), None, None, None, None, None, None, None, None, None, None);

        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), 3, db, fees);
        let keypair = Random.generate().unwrap();
        let address = public_to_address(keypair.public());

        test_client.set_balance(address, 1_000_000_000_000);

        let txs = vec![
            create_signed_pay_with_fee(0, 200, keypair),
            create_signed_pay_with_fee(1, 140, keypair),
            create_signed_pay_with_fee(2, 160, keypair),
        ];
        let result = abbreviated_mempool_add(&test_client, &mut mem_pool, txs, TxOrigin::Local);
        assert_eq!(
            vec![
                Ok(TransactionImportResult::Current),
                Ok(TransactionImportResult::Current),
                Ok(TransactionImportResult::Current)
            ],
            result
        );

        assert_eq!(
            vec![
                create_signed_pay_with_fee(0, 200, keypair),
                create_signed_pay_with_fee(1, 140, keypair),
                create_signed_pay_with_fee(2, 160, keypair)
            ],
            mem_pool.top_transactions(std::usize::MAX, None, 0..std::u64::MAX).transactions
        );

        assert_eq!(Vec::<SignedTransaction>::default(), mem_pool.future_transactions());
    }

    #[test]
    fn external_transactions_whose_fees_are_under_the_mem_pool_min_fee_are_rejected() {
        let test_client = TestBlockChainClient::new();
        // Set the pay transaction minimum fee
        let fees =
            MemPoolMinFees::create_from_options(Some(150), None, None, None, None, None, None, None, None, None, None);

        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), 3, db, fees);
        let keypair = Random.generate().unwrap();
        let address = public_to_address(keypair.public());

        test_client.set_balance(address, 1_000_000_000_000);

        let txs = vec![
            create_signed_pay_with_fee(0, 200, keypair),
            create_signed_pay_with_fee(1, 140, keypair),
            create_signed_pay_with_fee(1, 160, keypair),
            create_signed_pay_with_fee(2, 149, keypair),
        ];
        let result = abbreviated_mempool_add(&test_client, &mut mem_pool, txs, TxOrigin::External);
        assert_eq!(
            vec![
                Ok(TransactionImportResult::Current),
                Err(Error::Syntax(SyntaxError::InsufficientFee {
                    minimal: 150,
                    got: 140,
                })),
                Ok(TransactionImportResult::Current),
                Err(Error::Syntax(SyntaxError::InsufficientFee {
                    minimal: 150,
                    got: 149,
                })),
            ],
            result
        );

        assert_eq!(
            vec![create_signed_pay_with_fee(0, 200, keypair), create_signed_pay_with_fee(1, 160, keypair)],
            mem_pool.top_transactions(std::usize::MAX, None, 0..std::u64::MAX).transactions
        );

        assert_eq!(Vec::<SignedTransaction>::default(), mem_pool.future_transactions());
    }

    #[test]
    fn transactions_are_moved_to_future_queue_if_the_preceding_one_removed() {
        //setup test_client
        let test_client = TestBlockChainClient::new();

        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), 3, db, Default::default());

        let fetch_account = fetch_account_creator(&test_client);
        let keypair = Random.generate().unwrap();
        let address = public_to_address(keypair.public());
        println!("! {}", address);
        test_client.set_balance(address, 1_000_000_000_000);
        assert_eq!(1_000_000_000_000, test_client.latest_balance(&address));

        let inserted_block_number = 1;
        let inserted_timestamp = 100;
        let inputs = vec![
            create_mempool_input_with_pay(0, keypair),
            create_mempool_input_with_pay(1, keypair),
            create_mempool_input_with_pay(2, keypair),
        ];
        let result = mem_pool.add(inputs, inserted_block_number, inserted_timestamp, &fetch_account);
        assert_eq!(
            vec![
                Ok(TransactionImportResult::Current),
                Ok(TransactionImportResult::Current),
                Ok(TransactionImportResult::Current)
            ],
            result
        );

        assert_eq!(
            vec![create_signed_pay(0, keypair), create_signed_pay(1, keypair), create_signed_pay(2, keypair),],
            mem_pool.top_transactions(std::usize::MAX, None, 0..std::u64::MAX).transactions
        );

        assert_eq!(Vec::<SignedTransaction>::default(), mem_pool.future_transactions());

        let best_block_number = test_client.chain_info().best_block_number;
        let best_block_timestamp = test_client.chain_info().best_block_timestamp;
        let fetch_seq = |p: &Public| -> u64 {
            let address = public_to_address(p);
            test_client.latest_seq(&address)
        };
        mem_pool.remove(&[create_signed_pay(1, keypair).hash()], &fetch_seq, best_block_number, best_block_timestamp);

        assert_eq!(
            vec![create_signed_pay(0, keypair),],
            mem_pool.top_transactions(std::usize::MAX, None, 0..std::u64::MAX).transactions
        );

        assert_eq!(vec![create_signed_pay(2, keypair),], mem_pool.future_transactions());
    }
}
