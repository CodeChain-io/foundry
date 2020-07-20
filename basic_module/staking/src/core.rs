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

use crate::state::{Banned, Params};
use crate::transactions::Transaction;
use crate::types::Validator;
use coordinator::types::{ExecuteTransactionError, HeaderError, TransactionOutcome, VerifiedCrime};
use coordinator::Header;
use fkey::Ed25519Public as Public;
use std::collections::HashMap;

pub trait Abci {
    fn open_block(&self, header: &Header, verified_crime: &[VerifiedCrime]) -> Result<(), HeaderError>;
    fn execute_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<Vec<TransactionOutcome>, ExecuteTransactionError>;
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), i64>;
}

pub trait StakingView {
    fn get_stakes(&self) -> HashMap<Public, u64>;
    fn get_validators(&self) -> Vec<Validator>;
    fn current_term_id(&self) -> u64;
    fn get_term_common_params(&self) -> Params;
    fn is_term_changed(&self) -> bool;
    fn last_term_finished_block_num(&self) -> u64;
    fn era(&self) -> u64;
    fn get_banned_validators(&self) -> Banned;
}

pub trait AdditionalTxCreator {
    fn create(&self) -> Vec<Transaction>;
}
