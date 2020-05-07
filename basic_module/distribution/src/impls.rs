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

use crate::core::{BlockExecutor, FeeManager, TransactionHandler};
use crate::error::Error;
use crate::types::{Action, Transaction};
use crate::{
    add_balance, calculate_pending_rewards_of_the_term, check_network_id, current_term_id, fee_distribute,
    get_banned_validators, get_chain_history_access, get_intermediate_rewards, get_stakes, is_term_changed,
    last_term_finished_block_num, IntermediateRewards,
};
use ckey::Ed25519Public as Public;
use coordinator::context::SubStorageAccess;
use coordinator::types::{BlockOutcome, VerifiedCrime};
use ctypes::Header;
use parking_lot::RwLock;
use std::collections::BTreeMap;

#[allow(dead_code)]
struct Distribution {
    block_number: RwLock<u64>,
    block_author: RwLock<Public>,
}

struct Fee {}

impl FeeManager for Fee {
    fn accumulate_block_fee(&self, total_additional_fee: u64, total_min_fee: u64) {
        let mut intermediate_rewards: IntermediateRewards = get_intermediate_rewards();

        intermediate_rewards.total_additional_fee += total_additional_fee;
        intermediate_rewards.total_min_fee += total_min_fee;

        intermediate_rewards.save_to_state();
    }
}

impl TransactionHandler for Distribution {
    fn create_distribute_fee_transaction(&self) -> Transaction {
        let intermediate_rewards = get_intermediate_rewards();

        let block_reward: u64 = Default::default();
        let total_reward = intermediate_rewards.total_additional_fee + block_reward;
        let total_min_fee = intermediate_rewards.total_min_fee;
        let stakes = get_stakes();

        assert!(total_reward >= total_min_fee, "{} >= {}", total_reward, total_min_fee);

        let mut distributor = fee_distribute(total_min_fee, &stakes);
        let mut calculated_fees: BTreeMap<Public, u64> = BTreeMap::new();

        for (address, share) in &mut distributor {
            calculated_fees.insert(*address, share);
        }

        let block_author_rewards = total_reward - total_min_fee + distributor.remaining_fee();

        Transaction {
            seq: Default::default(),
            fee: Default::default(),
            network_id: Default::default(),
            order: 1,
            action: Action::DistributeRewards {
                calculated_fees,
                block_author_rewards,
            },
        }
    }

    fn create_distribute_rewards_transaction(&self) -> Transaction {
        let intermediate_rewards = get_intermediate_rewards();
        let rewards = intermediate_rewards.get_current_rewards();
        let block_number = *self.block_number.read();
        let block_author = *self.block_author.read();

        Transaction {
            seq: Default::default(),
            fee: Default::default(),
            network_id: Default::default(),
            order: 0,
            action: Action::UpdateRewards {
                block_number,
                block_author,
                rewards,
            },
        }
    }
}

impl Distribution {
    fn distribute_fee_to_stakeholders(
        &self,
        calculated_fees: BTreeMap<Public, u64>,
        block_author_rewards: u64,
    ) -> Result<(), Error> {
        let mut intermediate_rewards = get_intermediate_rewards();

        let block_author = *self.block_author.read();
        let block_number = *self.block_number.read();

        for (address, share) in calculated_fees.iter() {
            add_balance(&address, *share);
        }

        let term = current_term_id(block_number);
        match term {
            0 => {
                add_balance(&block_author, block_author_rewards);
            }
            _ => {
                intermediate_rewards.update_quantity(&block_author, block_author_rewards);
                intermediate_rewards.save_to_state();
            }
        }
        Ok(())
    }

    fn distribute_rewards_to_validators(
        &self,
        block_number: u64,
        _block_author: Public,
        rewards: BTreeMap<Public, u64>,
    ) -> Result<(), Error> {
        let mut intermediate_rewards = get_intermediate_rewards();
        let chain_history = get_chain_history_access();
        let term = current_term_id(block_number);

        if block_number == last_term_finished_block_num(block_number) + 1 {
            intermediate_rewards.update_current_rewards_to_empty()?;
            match term {
                0 | 1 => {}
                _ => {
                    let banned = get_banned_validators();
                    let start_of_the_current_term_header = chain_history
                        .get_block_header(block_number.into())
                        .ok_or(Error::InvalidNumber(block_number))?;

                    let pending_rewards = calculate_pending_rewards_of_the_term(
                        banned,
                        rewards,
                        start_of_the_current_term_header,
                        chain_history,
                    )?;
                    intermediate_rewards.calculated_reward = pending_rewards.into_iter().collect();
                    intermediate_rewards.save_to_state();
                }
            }
        }
        if is_term_changed(block_number) {
            for (address, reward) in intermediate_rewards.drain_calculated_rewards() {
                add_balance(&address, reward);
            }
        }
        Ok(())
    }
}

impl BlockExecutor for Distribution {
    fn open_block(&self, _context: &mut dyn SubStorageAccess, header: &Header, _verified_crime: &[VerifiedCrime]) {
        let mut author = self.block_author.write();
        let mut number = self.block_number.write();
        // We decided to use a public key as an address, and currently, this module
        // brings `block_author` that is Address type from the header in codechain-types.
        // We need to modify the type of the header to Public to use it as the address.
        // It wasn't updated yet, we implemented `block_author` as `Public::random()`.
        // let block_author = self.header.author();
        *author = Public::random();
        *number = header.number();
    }

    fn execute_transactions(
        &self,
        _context: &mut dyn SubStorageAccess,
        transactions: &[Transaction],
    ) -> Result<(), Error> {
        for tx in transactions {
            check_network_id(tx.network_id)?;
            match &tx.action {
                Action::DistributeRewards {
                    calculated_fees,
                    block_author_rewards,
                } => self
                    .distribute_fee_to_stakeholders(calculated_fees.clone(), *block_author_rewards)
                    .expect("Cannot execute a transaction that distributes rewards when a block is closing"),
                Action::UpdateRewards {
                    block_number,
                    block_author,
                    rewards,
                } => self
                    .distribute_rewards_to_validators(*block_number, *block_author, rewards.clone())
                    .expect("Cannot execute a transaction that open block for distributing rewards"),
            };
        }
        Ok(())
    }

    fn close_block(&self, _context: &mut dyn SubStorageAccess) -> BlockOutcome {
        unimplemented!();
    }
}
