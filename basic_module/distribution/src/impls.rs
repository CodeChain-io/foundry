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

use crate::core::{FeeManager, TransactionHandler};
use crate::types::{Action, Transaction};
use crate::{
    fee_distribute, get_intermediate_rewards, get_stakes, IntermediateRewards,
};
use ckey::Ed25519Public as Public;
use std::collections::BTreeMap;
use parking_lot::RwLock;

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
