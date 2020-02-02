// Copyright 2018-2020 Kodebox, Inc.
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
// A state machine.

use crate::block::{ExecutedBlock, IsBlock};
use crate::client::BlockChainTrait;
use crate::error::Error;
use crate::transaction::{SignedTransaction, UnverifiedTransaction};
use ckey::Address;
use cstate::{StateError, TopState, TopStateView};
use ctypes::errors::SyntaxError;
use ctypes::transaction::Action;
use ctypes::{CommonParams, Header};

pub struct CodeChainMachine {
    params: CommonParams,
}

impl CodeChainMachine {
    pub fn new(params: CommonParams) -> Self {
        CodeChainMachine {
            params,
        }
    }

    /// Get the general parameters of the chain.
    pub fn genesis_common_params(&self) -> &CommonParams {
        &self.params
    }

    /// Does basic verification of the transaction.
    pub fn verify_transaction_with_params(
        &self,
        tx: &UnverifiedTransaction,
        common_params: &CommonParams,
    ) -> Result<(), Error> {
        let min_cost = Self::min_cost(common_params, &tx.action);
        if tx.fee < min_cost {
            return Err(SyntaxError::InsufficientFee {
                minimal: min_cost,
                got: tx.fee,
            }
            .into())
        }
        tx.verify_with_params(common_params)?;

        Ok(())
    }

    /// Verify a particular transaction's seal is valid.
    pub fn verify_transaction_seal(p: UnverifiedTransaction, _header: &Header) -> Result<SignedTransaction, Error> {
        p.check_low_s()?;
        Ok(SignedTransaction::try_new(p)?)
    }

    /// Does verification of the transaction against the parent state.
    pub fn verify_transaction<C: BlockChainTrait>(
        &self,
        _tx: &SignedTransaction,
        _header: &Header,
        _client: &C,
        _verify_timelock: bool,
    ) -> Result<(), Error> {
        // FIXME: Filter transactions.
        Ok(())
    }

    /// Populate a header's fields based on its parent's header.
    /// Usually implements the chain scoring rule based on weight.
    pub fn populate_from_parent(&self, header: &mut Header, parent: &Header) {
        header.set_score(*parent.score());
    }

    pub fn min_cost(params: &CommonParams, action: &Action) -> u64 {
        match action {
            Action::Pay {
                ..
            } => params.min_pay_transaction_cost(),
            Action::SetRegularKey {
                ..
            } => params.min_set_regular_key_transaction_cost(),
            Action::CreateShard {
                ..
            } => params.min_create_shard_transaction_cost(),
            Action::SetShardOwners {
                ..
            } => params.min_set_shard_owners_transaction_cost(),
            Action::SetShardUsers {
                ..
            } => params.min_set_shard_users_transaction_cost(),
            Action::Custom {
                ..
            } => params.min_custom_transaction_cost(),
            Action::ShardStore {
                ..
            } => {
                // FIXME
                0
            }
        }
    }

    pub fn balance(&self, live: &ExecutedBlock, address: &Address) -> Result<u64, Error> {
        Ok(live.state().balance(address).map_err(StateError::from)?)
    }

    pub fn add_balance(&self, live: &mut ExecutedBlock, address: &Address, amount: u64) -> Result<(), Error> {
        live.state_mut().add_balance(address, amount).map_err(StateError::from)?;
        Ok(())
    }

    pub fn increase_term_id(&self, live: &mut ExecutedBlock, last_term_finished_block_num: u64) -> Result<(), Error> {
        live.state_mut().increase_term_id(last_term_finished_block_num)?;
        Ok(())
    }
}
