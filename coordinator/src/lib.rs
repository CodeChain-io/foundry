// Copyright 2020 Kodebox, Inc.
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

#![allow(dead_code, unused_variables)]
use self::traits::{BlockExecutor, Initializer, TxFilter};
use self::types::*;
use context::StorageAccess;
use ctypes::{CompactValidatorSet, ConsensusParams};

pub mod context;
pub mod test_coordinator;
pub mod traits;
pub mod types;

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.

#[derive(Default)]
pub struct Coordinator {}

impl Initializer for Coordinator {
    fn initialize_chain(&self, app_state: String) -> (CompactValidatorSet, ConsensusParams) {
        unimplemented!()
    }
}

impl BlockExecutor for Coordinator {
    fn open_block(
        &self,
        context: &mut dyn StorageAccess,
        header: &Header,
        verified_crime: &[VerifiedCrime],
    ) -> Result<(), HeaderError> {
        unimplemented!()
    }

    fn execute_transactions(
        &self,
        context: &mut dyn StorageAccess,
        transactions: &[Transaction],
    ) -> Result<Vec<TransactionExecutionOutcome>, ExecuteTransactionError> {
        unimplemented!()
    }

    fn prepare_block<'a>(
        &self,
        context: &mut dyn StorageAccess,
        transactions: Box<dyn Iterator<Item = &'a TransactionWithMetadata> + 'a>,
    ) -> Vec<&'a Transaction> {
        unimplemented!()
    }

    fn close_block(&self, context: &mut dyn StorageAccess) -> Result<BlockOutcome, CloseBlockError> {
        unimplemented!()
    }
}

impl TxFilter for Coordinator {
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn filter_transactions<'a>(
        &self,
        transactions: &'a [&'a TransactionWithMetadata],
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> (Vec<&'a TransactionWithMetadata>, Vec<&'a TransactionWithMetadata>) {
        unimplemented!()
    }
}

pub struct Builder<C: context::Context> {
    context: C,
}

impl<C: context::Context> Builder<C> {
    fn new(context: C) -> Self {
        Builder {
            context,
        }
    }

    fn build(self) -> Coordinator {
        Coordinator {}
    }
}
