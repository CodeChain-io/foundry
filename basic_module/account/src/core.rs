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

use crate::error::Error;
use crate::types::SignedTransaction;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public, Error as KeyError, Password, Signature};
pub use coordinator::context::SubStorageAccess;
pub use coordinator::types::{ErrorCode, TransactionOutcome};

pub trait CheckTxHandler {
    fn check_transaction(&self, tx: &SignedTransaction) -> Result<(), ErrorCode>;
}

pub trait TransactionExecutor {
    fn execute_transactions(&mut self, transactions: &[SignedTransaction]) -> Result<Vec<TransactionOutcome>, ()>;
}

pub trait AccountManager {
    fn add_balance(&mut self, account_id: &Public, val: u64);

    fn sub_balance(&mut self, account_id: &Public, val: u64) -> Result<(), Error>;

    fn set_balance(&mut self, account_id: &Public, val: u64);

    fn increment_sequence(&mut self, account_id: &Public);
}

pub trait AccountView {
    fn is_active(&self, account_id: &Public) -> bool;

    fn get_balance(&self, account_id: &Public) -> u64;

    fn get_sequence(&self, account_id: &Public) -> u64;
}
