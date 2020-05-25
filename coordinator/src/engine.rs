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
use crate::header::Header;
use crate::transaction::{Transaction, TransactionWithMetadata};
use crate::types::{BlockOutcome, CloseBlockError, ErrorCode, HeaderError, TransactionExecutionOutcome, VerifiedCrime};
use ctypes::{CompactValidatorSet, ConsensusParams};

pub trait Initializer: Send + Sync {
    fn initialize_chain(&self) -> (CompactValidatorSet, ConsensusParams);
}

pub trait BlockExecutor: Send + Sync {
    fn open_block(
        &self,
        storage: &mut dyn StorageAccess,
        header: &Header,
        verified_crimes: &[VerifiedCrime],
    ) -> Result<(), HeaderError>;
    fn execute_transactions(
        &self,
        storage: &mut dyn StorageAccess,
        transactions: &[Transaction],
    ) -> Result<Vec<TransactionExecutionOutcome>, ()>;
    fn prepare_block<'a>(
        &self,
        storage: &mut dyn StorageAccess,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
    ) -> Vec<&'a Transaction>;
    fn close_block(&self, storage: &mut dyn StorageAccess) -> Result<BlockOutcome, CloseBlockError>;
}

pub trait TxFilter: Send + Sync {
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;
    fn filter_transactions<'a>(
        &self,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> (Vec<&'a TransactionWithMetadata>, Vec<&'a TransactionWithMetadata>);
}
