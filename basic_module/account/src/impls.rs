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
use crate::{check_network_id, check_signature};
use ckey::Ed25519Public as Public;
use coordinator::context::Context;
use coordinator::types::{ErrorCode, TransactionExecutionOutcome};

pub struct Handler<C: Context> {
    context: C,
}

impl<C: Context> Handler<C> {
    #[allow(dead_code)]
    pub fn new(context: C) -> Self {
        Self {
            context,
        }
    }
}

const INVALID_NETWORK_ID: ErrorCode = 1;
const INVALID_SIGNATURE: ErrorCode = 2;
const INVALID_SEQ: ErrorCode = 0xFFFF_FFFF;

impl<C: Context> CheckTxHandler for Handler<C> {
    fn check_transaction(&self, signed_tx: &SignedTransaction) -> Result<(), ErrorCode> {
        if !check_network_id(&signed_tx.tx.network_id) {
            return Err(INVALID_NETWORK_ID)
        }
        if !check_signature(signed_tx) {
            return Err(INVALID_SIGNATURE)
        }

        let Action::Pay {
            sender,
            receiver: _,
            quantity: _,
        } = signed_tx.tx.action;
        if get_sequence(&self.context, &sender) > signed_tx.tx.seq {
            return Err(INVALID_SEQ)
        }

        Ok(())
    }
}

impl<C: Context> TransactionExecutor for Handler<C> {
    fn execute_transactions(
        &mut self,
        transactions: &[SignedTransaction],
    ) -> Result<Vec<TransactionExecutionOutcome>, ()> {
        for signed_tx in transactions {
            let Action::Pay {
                sender,
                receiver,
                quantity,
            } = signed_tx.tx.action;
            #[cfg(debug_assertions)]
            self.check_transaction(signed_tx).unwrap();
            if self.get_sequence(&sender) != signed_tx.tx.seq {
                return Err(())
            }

            if sub_balance(&mut self.context, &sender, signed_tx.tx.fee).is_err() {
                return Err(())
            }
            self.increment_sequence(&sender);
            self.context.create_checkpoint();

            if sub_balance(&mut self.context, &sender, quantity).is_err() {
                self.context.revert_to_the_checkpoint();
                continue
            }
            add_balance(&mut self.context, &receiver, quantity);
            self.context.discard_checkpoint();
        }

        Ok(vec![])
    }
}

impl<C: Context> AccountManager for Handler<C> {
    fn add_balance(&mut self, account_id: &Public, val: u64) {
        add_balance(&mut self.context, account_id, val)
    }

    fn sub_balance(&mut self, account_id: &Public, val: u64) -> Result<(), Error> {
        sub_balance(&mut self.context, account_id, val)
    }

    fn set_balance(&mut self, account_id: &Public, val: u64) {
        let mut account = get_account(&self.context, account_id);

        account.balance = val;
        self.context.set(account_id, account.to_vec());
    }

    fn increment_sequence(&mut self, account_id: &Public) {
        let mut account = get_account(&self.context, account_id);

        account.sequence += 1;
        self.context.set(account_id, account.to_vec());
    }
}

impl<C: Context> AccountView for Handler<C> {
    fn is_active(&self, account_id: &Public) -> bool {
        get_balance(&self.context, account_id) != 0 || get_sequence(&self.context, account_id) != 0
    }

    fn get_balance(&self, account_id: &Public) -> u64 {
        get_balance(&self.context, account_id)
    }

    fn get_sequence(&self, account_id: &Public) -> u64 {
        get_sequence(&self.context, account_id)
    }
}
