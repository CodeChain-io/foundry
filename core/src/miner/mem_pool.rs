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
use crate::transaction::PendingTransactions;
use crate::Error as CoreError;
use coordinator::validator::{ErrorCode, Transaction, TransactionWithMetadata, TxOrigin, Validator};
use ctypes::errors::{HistoryError, SyntaxError};
use ctypes::TxHash;
use kvdb::{DBTransaction, KeyValueDB};
use std::ops::{Deref, Range};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub enum Error {
    History(HistoryError),
    Syntax(SyntaxError),
    App(ErrorCode),
}

impl Error {
    pub fn into_core_error(self) -> CoreError {
        match self {
            Error::History(err) => CoreError::History(err),
            Error::Syntax(err) => CoreError::Syntax(err),
            Error::App(err_code) => {
                CoreError::Other(format!("Rejected by check_transaction with error code: {}", err_code))
            }
        }
    }
}

impl From<HistoryError> for Error {
    fn from(err: HistoryError) -> Error {
        Error::History(err)
    }
}

impl From<SyntaxError> for Error {
    fn from(err: SyntaxError) -> Error {
        Error::Syntax(err)
    }
}

pub struct MemPool {
    /// Coordinator used for checking incoming transactions and fetching transactions
    validator: Arc<dyn Validator>,
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
    pub fn with_limits(
        limit: usize,
        memory_limit: usize,
        db: Arc<dyn KeyValueDB>,
        validator: Arc<dyn Validator>,
    ) -> Self {
        MemPool {
            validator,
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
        let to_drop: Vec<TxHash> = {
            let transactions: Vec<_> = self.transaction_pool.pool.values().collect();
            let (invalid, low_priority) = self.validator.remove_transactions(
                &transactions,
                Some(self.queue_memory_limit),
                Some(self.queue_count_limit),
            );
            let to_drop_txes = [invalid, low_priority].concat();
            to_drop_txes.iter().map(|tx| tx.hash()).collect()
        };
        for hash in to_drop {
            backup::remove_item(batch, hash.deref());
            self.transaction_pool.remove(&hash);
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
    ) -> Vec<Result<(), Error>> {
        let mut batch = backup::backup_batch_with_capacity(transactions.len());
        let mut insert_results = Vec::with_capacity(transactions.len());
        for tx in transactions {
            let id = self.next_transaction_id;
            self.next_transaction_id += 1;

            let hash = tx.hash();
            match self.validator.check_transaction(&tx) {
                Ok(()) => {
                    let tx = TransactionWithMetadata::new(tx, origin, inserted_block_number, inserted_timestamp, id);
                    if self.transaction_pool.contains(&hash) {
                        // This transaction is already in the pool.
                        insert_results.push(Err(HistoryError::TransactionAlreadyImported.into()));
                    } else {
                        backup::backup_item(&mut batch, *tx.hash(), &tx);
                        self.transaction_pool.insert(tx);
                        insert_results.push(Ok(hash));
                    }
                }
                Err(err_code) => {
                    // This transaction is invalid.
                    insert_results.push(Err(Error::App(err_code)));
                }
            }
        }
        self.enforce_limit(&mut batch);

        self.db.write(batch).expect("Low level database error. Some issue with disk?");

        insert_results
            .into_iter()
            .map(|v| match v {
                Ok(hash) => {
                    if self.transaction_pool.contains(&hash) {
                        Ok(())
                    } else {
                        Err(HistoryError::LimitReached.into())
                    }
                }
                Err(e) => Err(e),
            })
            .collect()
    }

    pub fn remove_all(&mut self) {
        self.transaction_pool.clear();
    }

    // Recover MemPool state from db stored data
    pub fn recover_from_db(&mut self) {
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

    pub fn top_transactions(&self, gas_limit: usize, size_limit: usize, range: Range<u64>) -> PendingTransactions {
        let mut current_gas: usize = 0;
        let mut current_size: usize = 0;
        let unordered_transactions: Vec<_> =
            self.transaction_pool.pool.values().filter(|tx| range.contains(&tx.inserted_timestamp)).collect();
        let ordered_transactions = self.validator.fetch_transactions_for_block(&unordered_transactions);
        let chosen_transactions: Vec<_> = ordered_transactions
            .iter()
            .take_while(|tx_with_gas| {
                let size = tx_with_gas.size();
                let gas = tx_with_gas.gas;
                current_size += size;
                current_gas += gas;
                current_gas < gas_limit && current_size < size_limit
            })
            .collect();

        let last_timestamp =
            chosen_transactions.iter().map(|tx_with_gas| tx_with_gas.tx_with_metadata.inserted_timestamp).max();

        PendingTransactions {
            transactions: chosen_transactions
                .iter()
                .map(|tx_with_gas| tx_with_gas.tx_with_metadata.tx.clone())
                .collect(),
            last_timestamp,
        }
    }

    pub fn remove(&mut self, tx_hashes: &[TxHash]) {
        let mut batch = backup::backup_batch_with_capacity(tx_hashes.len());
        tx_hashes.iter().for_each(|hash| {
            self.transaction_pool.remove(hash);
            backup::remove_item(&mut batch, hash);
        });

        self.db.write(batch).expect("Low level database error. Some issue with disk?")
    }

    pub fn remove_old(&mut self) {
        let mut batch = backup::backup_batch_with_capacity(self.transaction_pool.count);
        let to_be_removed: Vec<TxHash> = {
            let transactions: Vec<_> = self.transaction_pool.pool.values().collect();
            let (invalid, low_priority) = self.validator.remove_transactions(&transactions, None, None);
            let transactions_to_be_removed = [invalid, low_priority].concat();
            transactions_to_be_removed.iter().map(|tx| tx.hash()).collect()
        };
        // TODO: mark invalid transactions
        for hash in to_be_removed {
            self.transaction_pool.remove(&hash);
            backup::remove_item(&mut batch, &hash);
        }

        self.db.write(batch).expect("Low level database error. Some issue with disk?")
    }

    pub fn count_pending_transactions(&self, range: Range<u64>) -> usize {
        self.transaction_pool.pool.values().filter(|tx| range.contains(&tx.inserted_timestamp)).count()
    }
}

#[cfg(test)]
pub mod test {
    use coordinator::test_coordinator::TestCoordinator;
    use coordinator::validator::Transaction;
    use rand::Rng;
    use std::sync::Arc;

    use super::*;

    fn create_random_transaction() -> Transaction {
        //FIXME: change this random to be reproducible
        let body_length = rand::thread_rng().gen_range(50, 200);
        let random_bytes = (0..body_length).map(|_| rand::random::<u8>()).collect();
        Transaction::new("Sample".to_string(), random_bytes)
    }

    #[test]
    fn remove_all() {
        let validator = Arc::new(TestCoordinator::default());
        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), db, validator);

        let inserted_block_number = 1;
        let inserted_timestamp = 100;
        let origin = TxOrigin::External;

        let transactions: Vec<_> = (0..10).map(|_| create_random_transaction()).collect();

        let add_result = mem_pool.add(transactions.clone(), origin, inserted_block_number, inserted_timestamp);
        assert!(add_result.iter().all(|r| r.is_ok()));

        mem_pool.remove_all();
        assert!(transactions.iter().all(|tx| { !mem_pool.transaction_pool.contains(&tx.hash()) }));
        assert_eq!(mem_pool.transaction_pool.len(), 0);
        assert_eq!(mem_pool.transaction_pool.count, 0);
        assert_eq!(mem_pool.transaction_pool.mem_usage, 0);
    }

    #[test]
    fn add_and_remove_transactions() {
        let validator = Arc::new(TestCoordinator::default());
        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), db, validator);

        let inserted_block_number = 1;
        let inserted_timestamp = 100;
        let origin = TxOrigin::External;

        let transactions: Vec<_> = (0..10).map(|_| create_random_transaction()).collect();

        let add_result = mem_pool.add(transactions.clone(), origin, inserted_block_number, inserted_timestamp);
        assert!(add_result.iter().all(|r| r.is_ok()));

        let (to_remove, to_keep) = transactions.split_at(5);

        let to_remove_hashes: Vec<_> = to_remove.iter().map(|tx| tx.hash()).collect();
        let to_keep_hashes: Vec<_> = to_keep.iter().map(|tx| tx.hash()).collect();
        mem_pool.remove(&to_remove_hashes);

        assert!(to_keep_hashes.iter().all(|hash| { mem_pool.transaction_pool.contains(hash) }));
        assert!(to_remove_hashes.iter().all(|hash| { !mem_pool.transaction_pool.contains(hash) }));

        let count: usize = 5;
        let mem_usage: usize = to_keep.iter().map(|tx| tx.size()).sum();

        assert_eq!(mem_pool.transaction_pool.count, count);
        assert_eq!(mem_pool.transaction_pool.mem_usage, mem_usage);
    }

    #[test]
    fn db_backup_and_recover() {
        let validator = Arc::new(TestCoordinator::default());
        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), db.clone(), validator.clone());

        let inserted_block_number = 1;
        let inserted_timestamp = 100;
        let origin = TxOrigin::External;

        let transactions: Vec<_> = (0..10).map(|_| create_random_transaction()).collect();

        let add_result = mem_pool.add(transactions, origin, inserted_block_number, inserted_timestamp);
        assert!(add_result.iter().all(|r| r.is_ok()));

        let inserted_block_number = 2;
        let inserted_timestamp = 200;
        let origin = TxOrigin::Local;

        let transactions: Vec<_> = (0..10).map(|_| create_random_transaction()).collect();

        let add_result = mem_pool.add(transactions, origin, inserted_block_number, inserted_timestamp);
        assert!(add_result.iter().all(|r| r.is_ok()));

        let mut mem_pool_recovered = MemPool::with_limits(8192, usize::max_value(), db, validator);
        mem_pool_recovered.recover_from_db();

        assert_eq!(mem_pool_recovered.transaction_pool, mem_pool.transaction_pool);
        assert_eq!(mem_pool_recovered.queue_count_limit, mem_pool.queue_count_limit);
        assert_eq!(mem_pool_recovered.queue_memory_limit, mem_pool.queue_memory_limit);
        assert_eq!(mem_pool_recovered.next_transaction_id, mem_pool.next_transaction_id);
    }
}
