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

use crate::core::FeeManager;
use crate::{get_intermediate_rewards,  IntermediateRewards};

struct Fee {}

impl FeeManager for Fee {
    fn accumulate_block_fee(&self, total_additional_fee: u64, total_min_fee: u64) {
        let mut intermediate_rewards: IntermediateRewards = get_intermediate_rewards();

        intermediate_rewards.total_additional_fee += total_additional_fee;
        intermediate_rewards.total_min_fee += total_min_fee;

        intermediate_rewards.save_to_state();
    }
}
