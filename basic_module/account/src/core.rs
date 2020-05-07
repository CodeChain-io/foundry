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

use crate::types::{SignedTransaction, Transaction};
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public, Error as KeyError, Password, Signature};
pub use coordinator::context::SubStorageAccess;
pub use coordinator::types::{ErrorCode, TransactionExecutionOutcome};

pub trait CheckTxHandler {
    fn check_transaction(&self, tx: &Transaction) -> Result<(), ErrorCode>;
}

pub trait TransactionExecutor {
    fn execute_transactions(&self, transactions: &[SignedTransaction]) -> Result<Vec<TransactionExecutionOutcome>, ()>;
}
