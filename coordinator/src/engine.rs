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
use crate::types::{
    BlockOutcome, CloseBlockError, ErrorCode, FilteredTxs, HeaderError, TransactionOutcome, VerifiedCrime,
};
use ctypes::{CompactValidatorSet, ConsensusParams};
use std::sync::Arc;

pub trait Initializer: Send + Sync {
    fn number_of_sub_storages(&self) -> usize;

    fn initialize_chain(&self, storage: &mut dyn StorageAccess) -> (CompactValidatorSet, ConsensusParams);
}

pub type ExecutionId = u32;

pub trait BlockExecutor: Send + Sync {
    fn open_block(
        &self,
        storage: &mut dyn StorageAccess,
        header: &Header,
        verified_crimes: &[VerifiedCrime],
    ) -> Result<ExecutionId, HeaderError>;
    fn execute_transactions(
        &self,
        execution_id: ExecutionId,
        storage: &mut dyn StorageAccess,
        transactions: &[Transaction],
    ) -> Result<Vec<TransactionOutcome>, ()>;
    fn prepare_block<'a>(
        &self,
        execution_id: ExecutionId,
        storage: &mut dyn StorageAccess,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
    ) -> Vec<(&'a Transaction, TransactionOutcome)>;
    fn close_block(&self, execution_id: ExecutionId) -> Result<BlockOutcome, CloseBlockError>;
}

pub trait TxFilter: Send + Sync {
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;
    fn filter_transactions<'a>(
        &self,
        storage: &mut dyn StorageAccess,
        transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> FilteredTxs<'a>;
}

pub trait GraphQlHandlerProvider: Send + Sync {
    /// Returns list of (module name, module graphql handler).
    fn get(&self) -> Vec<(String, Arc<dyn super::module::HandleGraphQlRequest>)>;

    fn new_session_for_query(&self, storage: &mut dyn StorageAccess) -> crate::module::SessionId;
    fn end_session_for_query(&self, session: crate::module::SessionId);
}
