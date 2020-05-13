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
use crate::account::{
    add_balance as add_balance_internalliy, get_account, get_balance as get_balance_internally,
    get_sequence as get_sequence_internally, sub_balance as sub_balance_internalliy,
};
use crate::import::{fee_manager, signature_manager};
use crate::{check, get_context, Action, SignedTransaction};
use ckey::{sign as sign_ed25519, verify as verify_ed25519};
use coordinator::context::SubStorageAccess;
pub use coordinator::types::ErrorCode;
pub use coordinator::types::TransactionExecutionOutcome;

#[allow(dead_code)]
pub struct Handler {}

impl CheckTxHandler for Handler {
    fn check_transaction(&self, tx: &Transaction) -> Result<(), ErrorCode> {
        if get_balance_internally(&tx.action.sender) < tx.fee + tx.action.quantity
            || get_sequence_internally(&tx.action.sender) <= tx.seq
        {
            return Err(-1)
        }
        Ok(())
    }
}

#[allow(dead_code)]
pub struct Executor {}

impl TransactionExecutor for Executor {
    fn execute_transactions(&self, transactions: &[SignedTransaction]) -> Result<Vec<TransactionExecutionOutcome>, ()> {
        let mut total_additional_fee: u64 = 0;
        let mut total_min_fee: u64 = 0;

        for signed_tx in transactions {
            total_additional_fee += signed_tx.tx.fee - signed_tx.tx.action.min_fee();
            total_min_fee += signed_tx.tx.action.min_fee();

            // FIXME: Suitable error handling is needed
            if !check(signature_manager(), signed_tx)
                || !sub_balance_internalliy(&signed_tx.tx.action.receiver, *signed_Tx.tx.action.quantity).is_ok()
            {
                return
            }
            add_balance_internalliy(&sender, signed_tx.tx.fee + quantity);
        }

        let fee_manager = fee_manager();
        fee_manager.accumulate_block_fee(total_additional_fee, total_min_fee);

        // TODO: Maybe we can return some event, if needed
        Ok(transactions
            .iter()
            .map(|_| TransactionExecutionOutcome {
                events: Default::default(),
            })
            .collect())
    }
}

#[allow(dead_code)]
pub struct Manager {}

impl AccountManager for Manager {
    fn add_balance(&self, address: &Public, val: u64) {
        add_balance_internalliy(address, val)
    }

    fn sub_balance(&self, address: &Public, val: u64) -> Result<(), Error> {
        sub_balance_internalliy(address, val)
    }

    fn set_balance(&self, address: &Public, val: u64) {
        let context = get_context();
        let mut account = get_account(address);

        account.balance = val;
        context.set(address, account.to_vec());
    }

    fn increment_sequence(&self, address: &Public) {
        let context = get_context();
        let mut account = get_account(address);

        account.sequence += 1;
        context.set(address, account.to_vec());
    }
}

#[allow(dead_code)]
pub struct View {}

impl AccountView for View {
    fn is_active(&self, address: &Public) -> bool {
        get_balance_internally(address) != 0 || get_sequence_internally(address) != 0
    }

    fn get_balance(&self, address: &Public) -> u64 {
        get_balance_internally(address)
    }

    fn get_sequence(&self, address: &Public) -> u64 {
        get_sequence_internally(address)
    }
}

#[allow(dead_code)]
pub struct SigManager {}

impl SignatureManager for SigManager {
    fn verify(&self, signature: &Signature, message: &[u8], public: &Public) -> bool {
        verify_ed25519(signature, message, public)
    }

    fn sign(&self, message: &[u8], private: &Private) -> Signature {
        sign_ed25519(message, private)
    }
}
