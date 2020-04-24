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

use crate::check::check;
use crate::core::{Abci, StakingView};
use crate::error::Error;
use crate::execute::{apply_internal, execute_auto_action};
use crate::state::{get_stakes, Banned, CurrentValidators, Metadata, Params};
use crate::transactions::{SignedTransaction, Transaction};
use crate::types::Validator;
use coordinator::types::{ExecuteTransactionError, HeaderError, TransactionExecutionOutcome, VerifiedCrime};
use coordinator::Header;
use fkey::Ed25519Public as Public;
use std::cell::RefCell;
use std::collections::HashMap;

struct ABCIHandle {
    executing_block_header: RefCell<Header>,
}

impl Abci for ABCIHandle {
    fn open_block(&self, header: &Header, _verified_crime: &[VerifiedCrime]) -> Result<(), HeaderError> {
        *self.executing_block_header.borrow_mut() = header.clone();
        Ok(())
    }

    fn execute_transactions(
        &self,
        transactions: Vec<Transaction>,
    ) -> Result<Vec<TransactionExecutionOutcome>, ExecuteTransactionError> {
        let results: Result<Vec<_>, _> = transactions
            .into_iter()
            .map(|tx| match tx {
                Transaction::User(signed_tx) => check(&signed_tx).map_err(Error::Syntax).and({
                    let SignedTransaction {
                        tx,
                        signer_public,
                        ..
                    } = signed_tx;
                    apply_internal(tx, &signer_public).map_err(Error::Runtime)
                }),
                Transaction::Auto(auto_action) => {
                    execute_auto_action(auto_action, self.executing_block_header.borrow().number())
                        .map_err(Error::Runtime)
                }
            })
            .collect();
        // TODO: handle errors
        results.map_err(|_| ())
    }

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), i64> {
        match transaction {
            Transaction::User(signed_tx) => check(signed_tx).map_err(|err| err.code()),
            Transaction::Auto(_) => Ok(()),
        }
    }
}

struct StakingViewer {}

impl StakingView for StakingViewer {
    fn get_stakes(&self) -> HashMap<Public, u64> {
        get_stakes()
    }

    fn get_validators(&self) -> Vec<Validator> {
        CurrentValidators::load().into()
    }

    fn current_term_id(&self) -> u64 {
        Metadata::load().current_term_id
    }

    fn get_term_common_params(&self) -> Params {
        Metadata::load().term_params
    }

    fn is_term_changed(&self) -> bool {
        unimplemented!()
    }

    fn last_term_finished_block_num(&self) -> u64 {
        Metadata::load().last_term_finished_block_num
    }

    fn era(&self) -> u64 {
        Metadata::load().term_params.era
    }

    fn get_banned_validators(&self) -> Banned {
        Banned::load()
    }
}
