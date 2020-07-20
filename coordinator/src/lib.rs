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

mod app_desc;
pub mod context;
pub mod engine;
mod header;
pub mod module;
pub mod test_coordinator;
mod transaction;
pub mod types;
mod values;

use self::context::{Context, StorageAccess};
use self::engine::{BlockExecutor, Initializer, TxFilter};
pub use self::header::Header;
use self::transaction::{Transaction, TransactionWithMetadata};
use self::types::{BlockOutcome, ErrorCode, VerifiedCrime};
use crate::engine::FilteredTxs;
use crate::types::{CloseBlockError, ExecuteTransactionError, HeaderError, TransactionOutcome};
use ctypes::{CompactValidatorSet, ConsensusParams};
use parking_lot::Mutex;
use std::sync::Arc;

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.
#[derive(Default)]
pub struct Coordinator {}

impl Initializer for Coordinator {
    fn initialize_chain(&self, _storage: Arc<Mutex<dyn StorageAccess>>) -> (CompactValidatorSet, ConsensusParams) {
        unimplemented!()
    }
}

impl BlockExecutor for Coordinator {
    fn open_block(
        &self,
        _storage: Arc<Mutex<dyn StorageAccess>>,
        _header: &Header,
        _verified_crimes: &[VerifiedCrime],
    ) -> Result<(), HeaderError> {
        unimplemented!()
    }

    fn execute_transactions(
        &self,
        _transactions: &[Transaction],
    ) -> Result<Vec<TransactionOutcome>, ExecuteTransactionError> {
        unimplemented!()
    }

    fn prepare_block<'a>(
        &self,
        _transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
    ) -> Vec<(&'a Transaction, TransactionOutcome)> {
        unimplemented!()
    }

    fn close_block(&self) -> Result<BlockOutcome, CloseBlockError> {
        unimplemented!()
    }
}

impl TxFilter for Coordinator {
    fn check_transaction(&self, _transaction: &Transaction) -> Result<(), ErrorCode> {
        unimplemented!()
    }

    fn filter_transactions<'a>(
        &self,
        _transactions: &mut dyn Iterator<Item = &'a TransactionWithMetadata>,
        _memory_limit: Option<usize>,
        _size_limit: Option<usize>,
    ) -> FilteredTxs<'a> {
        unimplemented!()
    }
}

pub struct Builder<C: Context> {
    _context: C,
}

impl<C: Context> Builder<C> {
    #[allow(dead_code)]
    fn new(context: C) -> Self {
        Builder {
            _context: context,
        }
    }

    #[allow(dead_code)]
    fn build(self) -> Coordinator {
        Coordinator {}
    }
}
