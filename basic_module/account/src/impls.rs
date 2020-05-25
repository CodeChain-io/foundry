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

use super::core::*;
use crate::error::Error;
use crate::internal::{
    add_balance as add_balance_internalliy, get_account, get_balance as get_balance_internally,
    get_sequence as get_sequence_internally, sub_balance as sub_balance_internalliy,
};
use crate::types::Action;
use crate::{check, get_context, SignedTransaction};
pub use coordinator::types::ErrorCode;
pub use coordinator::types::TransactionExecutionOutcome;

#[allow(dead_code)]
pub struct Handler {}

impl CheckTxHandler for Handler {
    fn check_transaction(&self, signed_tx: &SignedTransaction) -> Result<(), ErrorCode> {
        check(signed_tx);

        let Action::Pay {
            sender,
            receiver: _,
            quantity: _,
        } = signed_tx.tx.action;
        if get_sequence_internally(&sender) > signed_tx.tx.seq {
            return Err(-1)
        }

        Ok(())
    }
}

#[allow(dead_code)]
pub struct Executor {}

impl TransactionExecutor for Executor {
    fn execute_transactions(&self, transactions: &[SignedTransaction]) -> Result<Vec<TransactionExecutionOutcome>, ()> {
        for signed_tx in transactions {
            let Action::Pay {
                sender,
                receiver,
                quantity,
            } = signed_tx.tx.action;

            if !check(signed_tx) || sub_balance_internalliy(&receiver, quantity).is_err() {
                return Err(())
            }
            add_balance_internalliy(&sender, signed_tx.tx.fee + quantity);
        }

        Ok(vec![])
    }
}

#[allow(dead_code)]
pub struct Manager {}

impl AccountManager for Manager {
    fn add_balance(&self, account_id: &Public, val: u64) {
        add_balance_internalliy(account_id, val)
    }

    fn sub_balance(&self, account_id: &Public, val: u64) -> Result<(), Error> {
        sub_balance_internalliy(account_id, val)
    }

    fn set_balance(&self, account_id: &Public, val: u64) {
        let context = get_context();
        let mut account = get_account(account_id);

        account.balance = val;
        context.set(account_id, account.to_vec());
    }

    fn increment_sequence(&self, account_id: &Public) {
        let context = get_context();
        let mut account = get_account(account_id);

        account.sequence += 1;
        context.set(account_id, account.to_vec());
    }
}

#[allow(dead_code)]
pub struct View {}

impl AccountView for View {
    fn is_active(&self, account_id: &Public) -> bool {
        get_balance_internally(account_id) != 0 || get_sequence_internally(account_id) != 0
    }

    fn get_balance(&self, account_id: &Public) -> u64 {
        get_balance_internally(account_id)
    }

    fn get_sequence(&self, account_id: &Public) -> u64 {
        get_sequence_internally(account_id)
    }
}
