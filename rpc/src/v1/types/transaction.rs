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

use ccore::{LocalizedTransaction, PendingTransactions as PendingVerifiedTransactions};
use coordinator::Transaction as ValidatorTransaction;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Transaction {}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PendingTransactions {
    transactions: Vec<Transaction>,
    last_timestamp: Option<u64>,
}

impl From<PendingVerifiedTransactions> for PendingTransactions {
    fn from(_tx: PendingVerifiedTransactions) -> Self {
        unimplemented!()
    }
}

impl From<LocalizedTransaction> for Transaction {
    fn from(_p: LocalizedTransaction) -> Self {
        unimplemented!()
    }
}

impl From<ValidatorTransaction> for Transaction {
    fn from(_tx: ValidatorTransaction) -> Self {
        unimplemented!()
    }
}
