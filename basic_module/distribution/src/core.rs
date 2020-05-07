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
use crate::types::Transaction;
use coordinator::context::SubStorageAccess;
use coordinator::types::{BlockOutcome, VerifiedCrime};
use ctypes::Header;

pub trait FeeManager {
    fn accumulate_block_fee(&self, total_additional_fee: u64, total_min_fee: u64);
}

pub trait TransactionHandler {
    fn create_distribute_fee_transaction(&self) -> Transaction;

    fn create_distribute_rewards_transaction(&self) -> Transaction;
}

pub trait BlockExecutor {
    fn open_block(&self, context: &mut dyn SubStorageAccess, header: &Header, verified_crime: &[VerifiedCrime]);

    fn execute_transactions(
        &self,
        context: &mut dyn SubStorageAccess,
        transactions: &[Transaction],
    ) -> Result<(), Error>;

    fn close_block(&self, context: &mut dyn SubStorageAccess) -> BlockOutcome;
}
