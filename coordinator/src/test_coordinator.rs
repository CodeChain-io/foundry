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

use crate::context::StorageAccess;
use crate::engine::{BlockExecutor, Initializer, TxFilter};
use crate::header::Header;
use crate::transaction::{Transaction, TransactionWithMetadata};
use crate::types::{
    BlockOutcome, CloseBlockError, ErrorCode, ExecuteTransactionError, HeaderError, TransactionExecutionOutcome,
    VerifiedCrime,
};
use ctypes::{CompactValidatorSet, ConsensusParams};
use parking_lot::Mutex;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

// Coordinator dedicated for mempool and miner testing
pub struct TestCoordinator {
    validator_set: CompactValidatorSet,
    consensus_params: ConsensusParams,
    body_count: AtomicUsize,
    body_size: AtomicUsize,
}

impl Default for TestCoordinator {
    fn default() -> Self {
        Self {
            validator_set: Default::default(),
            consensus_params: ConsensusParams::default_for_test(),
            body_count: AtomicUsize::new(0),
            body_size: AtomicUsize::new(0),
        }
    }
}

impl Initializer for TestCoordinator {
    fn initialize_chain(&self, _storage: Arc<Mutex<dyn StorageAccess>>) -> (CompactValidatorSet, ConsensusParams) {
        (self.validator_set.clone(), self.consensus_params)
    }
}

impl BlockExecutor for TestCoordinator {
    fn open_block(
        &self,
        _storage: Arc<Mutex<dyn StorageAccess>>,
        _header: &Header,
        _verified_crime: &[VerifiedCrime],
    ) -> Result<(), HeaderError> {
        self.body_count.store(0, Ordering::SeqCst);
        self.body_size.store(0, Ordering::SeqCst);
        Ok(())
    }

    fn execute_transactions(
        &self,
        transactions: &[Transaction],
    ) -> Result<Vec<TransactionExecutionOutcome>, ExecuteTransactionError> {
        self.body_count.fetch_add(transactions.len(), Ordering::SeqCst);
        let body_size: usize = transactions.iter().map(|tx| tx.size()).sum();
        self.body_size.fetch_add(body_size, Ordering::SeqCst);
        Ok((0..self.body_count.load(Ordering::SeqCst))
            .map(|_| TransactionExecutionOutcome {
                events: Vec::new(),
            })
            .collect())
    }

    fn prepare_block<'a>(
        &self,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
    ) -> Vec<&'a Transaction> {
        transactions.map(|tx_with_metadata| &tx_with_metadata.tx).collect()
    }

    fn close_block(&self) -> Result<BlockOutcome, CloseBlockError> {
        if self.body_size.load(Ordering::SeqCst) > self.consensus_params.max_body_size() as usize {
            Ok(BlockOutcome {
                updated_validator_set: Some(self.validator_set.clone()),
                updated_consensus_params: Some(self.consensus_params),

                events: Vec::new(),
            })
        } else {
            Err(String::from("Block size exceeds the maximum value"))
        }
    }
}

impl TxFilter for TestCoordinator {
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode> {
        if transaction.size() > self.consensus_params.max_body_size() as usize {
            Err(1)
        } else {
            Ok(())
        }
    }

    fn filter_transactions<'a>(
        &self,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> (Vec<&'a TransactionWithMetadata>, Vec<&'a TransactionWithMetadata>) {
        let invalid = Vec::new();
        let mut memory = 0;
        let mut size = 0;
        let low_priority = transactions
            .skip_while(|tx| {
                memory += (*tx).size();
                size += 1;
                memory <= memory_limit.unwrap_or(usize::max_value()) && size <= size_limit.unwrap_or(usize::max_value())
            })
            .collect();
        (invalid, low_priority)
    }
}
