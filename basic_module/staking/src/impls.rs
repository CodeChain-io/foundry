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
use crate::core::{
    Abci, ExecuteTransactionError, HeaderError, StakingView, TransactionExecutionOutcome, VerifiedCrime,
};
use crate::error::Error;
use crate::execute::apply_internal;
use crate::fee_manager;
use crate::state::{get_stakes, Banned, CurrentValidators, Metadata, Params};
use crate::transactions::SignedTransaction;
use crate::types::{Header, Public, ResultantFee, Validator};
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
        transactions: Vec<SignedTransaction>,
    ) -> Result<Vec<TransactionExecutionOutcome>, ExecuteTransactionError> {
        let results: Result<Vec<_>, _> = transactions
            .into_iter()
            .map(|signed_tx| {
                check(&signed_tx).map_err(Error::Syntax).and({
                    let SignedTransaction {
                        tx,
                        signer_public,
                        ..
                    } = signed_tx;
                    apply_internal(tx, &signer_public, self.executing_block_header.borrow().number())
                        .map_err(Error::Runtime)
                })
            })
            .collect();

        // failed block does not accumulate fee and rejected
        results
            .map(|results| {
                let (outcomes, fees): (_, Vec<_>) = results.into_iter().unzip();
                let ResultantFee {
                    additional_fee: total_additional_fee,
                    min_fee: total_min_fee,
                } = fees.into_iter().fold(ResultantFee::default(), |fee_acc, fee| fee_acc + fee);
                fee_manager().accumulate_block_fee(total_additional_fee, total_min_fee);
                outcomes
            })
            .map_err(|_| ())
    }

    fn check_transaction(&self, transaction: &SignedTransaction) -> Result<(), i64> {
        check(transaction).map_err(|err| err.code())
    }
}

struct StakingViewer {}

impl StakingView for StakingViewer {
    fn get_stakes(&self) -> HashMap<Public, u64> {
        get_stakes()
    }

    fn last_term_finished_block_num(&self) -> u64 {
        Metadata::load().last_term_finished_block_num
    }

    fn get_term_common_params(&self) -> Params {
        Metadata::load().term_params
    }

    fn era(&self) -> u64 {
        Metadata::load().term_params.era
    }

    fn is_term_changed(&self) -> bool {
        unimplemented!()
    }

    fn current_term_id(&self) -> u64 {
        Metadata::load().current_term_id
    }

    fn get_validators(&self) -> Vec<Validator> {
        CurrentValidators::load().into()
    }

    fn get_banned_validators(&self) -> Banned {
        Banned::load()
    }
}
