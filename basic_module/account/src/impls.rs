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

use crate::core::{AccountManager, AccountView, CheckTxHandler, TransactionExecutor};
use crate::error::Error;
use crate::internal::{add_balance, get_account, get_balance, get_sequence, sub_balance};
use crate::types::{Action, SignedTransaction};
use crate::{check, get_context};
use ckey::Ed25519Public as Public;
use coordinator::types::{ErrorCode, TransactionExecutionOutcome};

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
        if get_sequence(&sender) > signed_tx.tx.seq {
            return Err(0xFFFF_FFFF)
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

            if !check(signed_tx) || sub_balance(&receiver, quantity).is_err() {
                return Err(())
            }
            add_balance(&sender, signed_tx.tx.fee + quantity);
        }

        Ok(vec![])
    }
}

#[allow(dead_code)]
pub struct Manager {}

impl AccountManager for Manager {
    fn add_balance(&self, account_id: &Public, val: u64) {
        add_balance(account_id, val)
    }

    fn sub_balance(&self, account_id: &Public, val: u64) -> Result<(), Error> {
        sub_balance(account_id, val)
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
        get_balance(account_id) != 0 || get_sequence(account_id) != 0
    }

    fn get_balance(&self, account_id: &Public) -> u64 {
        get_balance(account_id)
    }

    fn get_sequence(&self, account_id: &Public) -> u64 {
        get_sequence(account_id)
    }
}
