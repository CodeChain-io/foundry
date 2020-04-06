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

use super::backup;
use super::mem_pool_types::{MemPoolItem, PoolingInstant, TransactionPool};
use super::TransactionImportResult;
use crate::transaction::{PendingVerifiedTransactions, VerifiedTransaction};
use crate::Error as CoreError;
use coordinator::engine::TxFilter;
use coordinator::TxOrigin;
use ctypes::errors::{HistoryError, RuntimeError, SyntaxError};
use ctypes::TxHash;
use kvdb::{DBTransaction, KeyValueDB};
use std::ops::Range;
use std::sync::Arc;

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
    /// Coordinator used for checking incoming transactions and fetching transactions
    _tx_filter: Arc<dyn TxFilter>,
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
        tx_filter: Arc<dyn TxFilter>,
    ) -> Self {
        MemPool {
            _tx_filter: tx_filter,
            transaction_pool: TransactionPool::new(),
            queue_count_limit: limit,
            queue_memory_limit: memory_limit,
            next_transaction_id: 0,
            db,
        }
    }

    /// Set the new limit for the `current` queue.
    pub fn set_limit(&mut self, limit: usize) {
        self.queue_count_limit = limit;
    }

    /// Enforce the limit to the current queue
    fn enforce_limit(&mut self, batch: &mut DBTransaction) {
        let to_drop = if self.transaction_pool.mem_usage > self.queue_memory_limit
            || self.transaction_pool.count > self.queue_count_limit
        {
            let mut count = 0;
            let mut mem_usage = 0;
            self.transaction_pool
                .pool
                .values()
                .filter(|item| {
                    count += 1;
                    mem_usage += item.mem_usage;
                    !item.origin.is_local() && (mem_usage > self.queue_memory_limit || count > self.queue_count_limit)
                })
                .cloned()
                .collect()
        } else {
            vec![]
        };

        for item in to_drop {
            let hash = item.hash();
            backup::remove_item(batch, &hash);
            self.transaction_pool.remove(&hash);
        }
    }

    /// Returns current limit of transactions in the pool.
    pub fn limit(&self) -> usize {
        self.queue_count_limit
    }

    /// Returns the number of transactions in the pool
    pub fn num_pending_transactions(&self) -> usize {
        self.transaction_pool.len()
    }

    /// Add signed transaction to pool to be verified and imported.
    ///
    /// NOTE details_provider methods should be cheap to compute
    /// otherwise it might open up an attack vector.
    pub fn add(
        &mut self,
        transactions: Vec<VerifiedTransaction>,
        origin: TxOrigin,
        inserted_block_number: PoolingInstant,
        inserted_timestamp: u64,
    ) -> Vec<Result<TransactionImportResult, Error>> {
        ctrace!(MEM_POOL, "add() called, time: {}, timestamp: {}", inserted_block_number, inserted_timestamp);
        let mut insert_results = Vec::new();
        let mut batch = backup::backup_batch_with_capacity(transactions.len());

        for tx in transactions {
            let hash = tx.hash();

            if let Err(e) = self.verify_transaction(&tx) {
                insert_results.push(Err(e));
                continue
            }

            let id = self.next_transaction_id;
            self.next_transaction_id += 1;
            let item = MemPoolItem::new(tx, origin, inserted_block_number, inserted_timestamp, id);

            backup::backup_item(&mut batch, *hash, &item);
            self.transaction_pool.insert(item);

            insert_results.push(Ok(()));
        }

        self.enforce_limit(&mut batch);

        self.db.write(batch).expect("Low level database error. Some issue with disk?");
        insert_results
            .into_iter()
            .map(|v| match v {
                Ok(_) => Ok(TransactionImportResult::Current),
                Err(e) => Err(e),
            })
            .collect()
    }

    /// Clear current queue.
    pub fn remove_all(&mut self) {
        self.transaction_pool.clear();
    }

    // Recover MemPool state from db stored data
    pub fn recover_from_db(&mut self) {
        let by_hash = backup::recover_to_data(self.db.as_ref());

        let mut max_insertion_id = 0u64;
        for (_hash, item) in by_hash {
            if item.insertion_id > max_insertion_id {
                max_insertion_id = item.insertion_id;
            }

            self.transaction_pool.insert(item);
        }

        self.next_transaction_id = max_insertion_id + 1;
    }

    /// Removes invalid transaction identified by hash from pool.
    /// Assumption is that this transaction seq is not related to client seq,
    /// so transactions left in pool are processed according to client seq.
    pub fn remove(
        &mut self,
        transaction_hashes: &[TxHash],
        current_block_number: PoolingInstant,
        current_timestamp: u64,
    ) {
        ctrace!(MEM_POOL, "remove() called, time: {}, timestamp: {}", current_block_number, current_timestamp);
        let mut batch = backup::backup_batch_with_capacity(transaction_hashes.len());

        for hash in transaction_hashes {
            if self.transaction_pool.remove(hash) {
                backup::remove_item(&mut batch, hash);
            }
        }

        self.db.write(batch).expect("Low level database error. Some issue with disk?");
    }

    /// Verify signed transaction with its content.
    /// This function can return errors: InsufficientFee, InsufficientBalance,
    /// TransactionAlreadyImported, Old, TooCheapToReplace
    fn verify_transaction(&self, tx: &VerifiedTransaction) -> Result<(), Error> {
        if self.transaction_pool.contains(&tx.hash()) {
            ctrace!(MEM_POOL, "Dropping already imported transaction: {:?}", tx.hash());
            return Err(HistoryError::TransactionAlreadyImported.into())
        }

        Ok(())
    }

    /// Returns top transactions whose timestamp are in the given range from the pool ordered by priority.
    // FIXME: current_timestamp should be `u64`, not `Option<u64>`.
    // FIXME: if range_contains becomes stable, use range.contains instead of inequality.
    pub fn pending_transactions(&self, size_limit: usize, range: Range<u64>) -> PendingVerifiedTransactions {
        let mut current_size: usize = 0;
        let items: Vec<_> = self
            .transaction_pool
            .pool
            .values()
            .filter(|item| range.contains(&item.inserted_timestamp))
            .take_while(|item| {
                let encoded_byte_array = rlp::encode(&item.tx);
                let size_in_byte = encoded_byte_array.len();
                current_size += size_in_byte;
                current_size < size_limit
            })
            .collect();

        let last_timestamp = items.iter().map(|t| t.inserted_timestamp).max();

        PendingVerifiedTransactions {
            transactions: items.into_iter().map(|t| t.tx.clone()).collect(),
            last_timestamp,
        }
    }

    /// Return all transactions whose timestamp are in the given range in the memory pool.
    pub fn count_pending_transactions(&self, range: Range<u64>) -> usize {
        self.transaction_pool.pool.values().filter(|t| range.contains(&t.inserted_timestamp)).count()
    }
}

#[cfg(test)]
pub mod test {
    use crate::transaction::UnverifiedTransaction;
    use std::cmp::Ordering;

    use ckey::{Ed25519KeyPair as KeyPair, Generator, KeyPairTrait, Random};
    use ctypes::transaction::Transaction;

    use super::backup::MemPoolItemProjection;
    use super::*;
    use coordinator::test_coordinator::TestCoordinator;
    use rlp::{rlp_encode_and_decode_test, Rlp};
    use std::convert::TryInto;

    #[test]
    fn origin_ordering() {
        assert_eq!(TxOrigin::Local.cmp(&TxOrigin::External), Ordering::Less);
        assert_eq!(TxOrigin::External.cmp(&TxOrigin::Local), Ordering::Greater);
    }

    #[test]
    fn txorigin_encode_and_decode() {
        rlp_encode_and_decode_test!(TxOrigin::External);
    }

    #[test]
    fn signed_transaction_encode_and_decode() {
        let keypair: KeyPair = Random.generate().unwrap();
        let tx = Transaction {
            network_id: "tc".into(),
        };
        let signed = VerifiedTransaction::new_with_sign(tx, keypair.private());

        let rlp = rlp::encode(&signed);
        let encoded = Rlp::new(&rlp);
        let decoded: UnverifiedTransaction = encoded.as_val().unwrap();
        let result = decoded.try_into().unwrap();

        assert_eq!(signed, result);
    }

    #[test]
    fn mempool_item_encode_and_decode() {
        let keypair: KeyPair = Random.generate().unwrap();
        let tx = Transaction {
            network_id: "tc".into(),
        };
        let signed = VerifiedTransaction::new_with_sign(tx, keypair.private());
        let item = MemPoolItem::new(signed, TxOrigin::Local, 0, 0, 0);

        let rlp = rlp::encode(&item);
        let encoded = Rlp::new(&rlp);
        let decoded: MemPoolItemProjection = encoded.as_val().unwrap();
        let result = decoded.try_into().unwrap();

        assert_eq!(item, result);
    }

    #[test]
    fn db_backup_and_recover() {
        //setup test_client
        let keypair: KeyPair = Random.generate().unwrap();

        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let coordinator = Arc::new(TestCoordinator::default());
        let mut mem_pool = MemPool::with_limits(8192, usize::max_value(), db.clone(), coordinator.clone());

        let inserted_block_number = 1;
        let inserted_timestamp = 100;
        let mut inputs: Vec<VerifiedTransaction> = Vec::new();

        inputs.push(create_signed_pay(&keypair));
        inputs.push(create_signed_pay(&keypair));
        inputs.push(create_signed_pay(&keypair));
        mem_pool.add(inputs, TxOrigin::Local, inserted_block_number, inserted_timestamp);

        let inserted_block_number = 11;
        let inserted_timestamp = 200;
        let mut inputs: Vec<_> = Vec::new();
        inputs.push(create_signed_pay(&keypair));
        inputs.push(create_signed_pay(&keypair));
        mem_pool.add(inputs, TxOrigin::Local, inserted_block_number, inserted_timestamp);

        let inserted_block_number = 20;
        let inserted_timestamp = 300;
        let mut inputs: Vec<_> = Vec::new();
        inputs.push(create_signed_pay(&keypair));
        inputs.push(create_signed_pay(&keypair));
        inputs.push(create_signed_pay(&keypair));
        mem_pool.add(inputs, TxOrigin::Local, inserted_block_number, inserted_timestamp);

        let inserted_block_number = 21;
        let inserted_timestamp = 400;
        let mut inputs: Vec<_> = Vec::new();
        inputs.push(create_signed_pay(&keypair));
        mem_pool.add(inputs, TxOrigin::Local, inserted_block_number, inserted_timestamp);

        let mut mem_pool_recovered = MemPool::with_limits(8192, usize::max_value(), db, coordinator);
        mem_pool_recovered.recover_from_db();

        assert_eq!(mem_pool_recovered.next_transaction_id, mem_pool.next_transaction_id);
        assert_eq!(mem_pool_recovered.queue_count_limit, mem_pool.queue_count_limit);
        assert_eq!(mem_pool_recovered.queue_memory_limit, mem_pool.queue_memory_limit);
        assert_eq!(mem_pool_recovered.transaction_pool, mem_pool.transaction_pool);
    }

    fn create_signed_pay(keypair: &KeyPair) -> VerifiedTransaction {
        let tx = Transaction {
            network_id: "tc".into(),
        };
        VerifiedTransaction::new_with_sign(tx, keypair.private())
    }
}
