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

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

mod core;
mod error;
mod impls;
mod import;
mod types;

use coordinator::context::Context;
use error::Error;
use import::{
    add_balance, current_term_id, get_banned_validators, get_chain_history_access, get_current_validators,
    get_previous_validators, get_stakes, is_term_changed, last_term_finished_block_num, Banned, ChainHistoryAccess,
};
use num_rational::Ratio;
use parking_lot::Mutex;
use primitives::Bytes;
use std::collections::{hash_map, BTreeMap, HashMap};
use std::convert::{From, TryFrom};
use std::mem;

pub use ckey::{Ed25519Public as Public, NetworkId};
pub use ctypes::{BlockHash, BlockNumber, CommonParams, Header};
use std::unimplemented;
pub use types::{BitSet, BlockId};

pub type StakeQuantity = u64;
pub type Deposit = u64;
pub type StorageId = u16;

pub const CUSTOM_ACTION_HANDLER_ID: u64 = 2;

#[derive(Clone, Debug)]
pub struct IntermediateRewards {
    current_reward: BTreeMap<Public, u64>,
    calculated_reward: BTreeMap<Public, u64>,
    total_additional_fee: u64,
    total_min_fee: u64,
}

impl IntermediateRewards {
    pub fn save_to_state(&self) {
        let context = get_context();

        context.set(
            b"Intermediate Rewards",
            Self {
                current_reward: self.current_reward.clone(),
                calculated_reward: self.calculated_reward.clone(),
                total_additional_fee: self.total_additional_fee,
                total_min_fee: self.total_min_fee,
            }
            .into(),
        );
    }

    pub fn update_quantity(&mut self, address: &Public, quantity: u64) {
        if quantity == 0 {
            return
        }
        *self.current_reward.entry(*address).or_insert(0) = quantity;
    }

    pub fn drain_calculated_rewards(&mut self) -> BTreeMap<Public, u64> {
        let mut drained = BTreeMap::new();
        mem::swap(&mut drained, &mut self.calculated_reward);

        self.save_to_state();
        drained
    }

    pub fn update_current_rewards_to_empty(&mut self) -> Result<(), Error> {
        let drained = BTreeMap::new();
        self.current_reward = drained;

        self.save_to_state();
        Ok(())
    }

    pub fn get_current_rewards(&self) -> BTreeMap<Public, u64> {
        self.current_reward.clone()
    }
}

impl Default for IntermediateRewards {
    fn default() -> Self {
        IntermediateRewards {
            current_reward: BTreeMap::default(),
            calculated_reward: BTreeMap::default(),
            total_additional_fee: Default::default(),
            total_min_fee: Default::default(),
        }
    }
}

impl From<Vec<u8>> for IntermediateRewards {
    fn from(_vec: Vec<u8>) -> Self {
        IntermediateRewards::default()
    }
}

impl From<IntermediateRewards> for Vec<u8> {
    fn from(_reward: IntermediateRewards) -> Self {
        Vec::default()
    }
}

pub struct FeeDistributeIter<'a> {
    total_stakes: u64,
    total_min_fee: u64,
    remaining_fee: u64,
    stake_holdings: hash_map::Iter<'a, Public, u64>,
}

impl<'a> FeeDistributeIter<'a> {
    pub fn remaining_fee(&self) -> u64 {
        self.remaining_fee
    }
}

pub fn fee_distribute<S: ::std::hash::BuildHasher>(
    total_min_fee: u64,
    stakes: &HashMap<Public, u64, S>,
) -> FeeDistributeIter<'_> {
    FeeDistributeIter {
        total_stakes: stakes.values().sum(),
        total_min_fee,
        remaining_fee: total_min_fee,
        stake_holdings: stakes.iter(),
    }
}

impl<'a> Iterator for FeeDistributeIter<'a> {
    type Item = (&'a Public, u64);
    fn next(&mut self) -> Option<(&'a Public, u64)> {
        if let Some((stakeholder, stake)) = self.stake_holdings.next() {
            let share = share(self.total_stakes, *stake, self.total_min_fee);
            self.remaining_fee = self.remaining_fee.checked_sub(share).expect("Remaining fee shouldn't be depleted");
            Some((stakeholder, share))
        } else {
            None
        }
    }
}

fn share(total_stakes: u64, stake: u64, total_min_fee: u64) -> u64 {
    assert!(total_stakes >= stake);
    u64::try_from((u128::from(total_min_fee) * u128::from(stake)) / u128::from(total_stakes)).unwrap()
}

pub fn get_context() -> &'static mut dyn Context {
    // This function should be implemented after the context has been formatted.
    unimplemented!();
}

pub fn get_intermediate_rewards() -> IntermediateRewards {
    get_context().get(b"Intermediate Rewards").map(|vec| vec.into()).unwrap_or_default()
}

/// reward = floor(intermediate_rewards * (a * number_of_signatures / number_of_blocks_in_term + b) / 10)
pub fn final_rewards(intermediate_reward: u64, number_of_signatures: u64, number_of_blocks_in_term: u64) -> u64 {
    let (a, b) = if number_of_signatures * 3 >= number_of_blocks_in_term * 2 {
        // number_of_signatures / number_of_blocks_in_term >= 2 / 3
        // x * 3/10 + 7/10
        (3, 7)
    } else if number_of_signatures * 2 >= number_of_blocks_in_term {
        // number_of_signatures / number_of_blocks_in_term >= 1 / 2
        // x * 48/10 - 23/10
        (48, -23)
    } else if number_of_signatures * 3 >= number_of_blocks_in_term {
        // number_of_signatures / number_of_blocks_in_term >= 1 / 3
        // x * 6/10 - 2/10
        (6, -2)
    } else {
        // 1 / 3 > number_of_signatures / number_of_blocks_in_term
        // 0
        assert!(
            number_of_blocks_in_term > 3 * number_of_signatures,
            "number_of_signatures / number_of_blocks_in_term = {}",
            (number_of_signatures as f64) / (number_of_blocks_in_term as f64)
        );
        (0, 0)
    };
    let numerator = i128::from(intermediate_reward)
        * (a * i128::from(number_of_signatures) + b * i128::from(number_of_blocks_in_term));
    assert!(numerator >= 0);
    let denominator = 10 * i128::from(number_of_blocks_in_term);
    // Rust's division rounds towards zero.
    u64::try_from(numerator / denominator).unwrap()
}

#[allow(dead_code)]
#[derive(Default)]
struct WorkInfo {
    proposed: usize,
    missed: usize,
    signed: u64,
}

fn aggregate_work_info(
    start_of_the_next_term_header: Header,
    chain_history: Box<dyn ChainHistoryAccess>,
) -> HashMap<Public, WorkInfo> {
    let mut work_info = HashMap::<Public, WorkInfo>::new();

    let start_of_the_current_term = {
        let end_of_the_last_term = last_term_finished_block_num(start_of_the_next_term_header.number() - 2);
        end_of_the_last_term + 1
    };

    let mut header = start_of_the_next_term_header;
    while start_of_the_current_term != header.number() {
        let parent_header = chain_history.get_block_header(header.number().into()).unwrap();
        let current_validators = get_current_validators(parent_header.hash().into());
        let previous_validators = get_previous_validators(parent_header.hash().into());
        for index in get_bitset(header.seal()).true_index_iter() {
            let signer = *current_validators.get(index).expect("The seal must be the signature of the validator");
            work_info.entry(signer).or_default().signed += 1;
        }

        // We decided to use a public key as an address, and currently, this module
        // brings `author` that is Address type from the header in codechain-types.
        // We need to modify the type of the header to Public to use it as the address.
        // It wasn't updated yet, we implemented `author` as `Public::random()`.
        // let author = parent_header.author();
        let author = Public::random();
        let info = work_info.entry(author).or_default();
        info.proposed += 1;
        info.missed += previous_validators.len() - get_bitset(header.seal()).count();

        header = parent_header;
    }
    work_info
}

fn calculate_pending_rewards_of_the_term(
    banned: Banned,
    rewards: BTreeMap<Public, u64>,
    start_of_the_next_term_header: Header,
    chain_history: Box<dyn ChainHistoryAccess>,
) -> Result<HashMap<Public, u64>, Error> {
    const MAX_NUM_OF_VALIDATORS: usize = 30;
    let work_info = aggregate_work_info(start_of_the_next_term_header, chain_history);
    let mut pending_rewards = HashMap::<Public, u64>::with_capacity(MAX_NUM_OF_VALIDATORS);

    let mut reduced_rewards = 0;

    let number_of_blocks_in_term: usize = work_info.values().map(|info| info.proposed).sum();
    for (address, intermediate_reward) in rewards {
        if banned.is_banned(&address) {
            reduced_rewards += intermediate_reward;
            continue
        }
        let number_of_signatures = work_info.get(&address).unwrap().signed;
        let final_block_rewards =
            final_rewards(intermediate_reward, number_of_signatures, u64::try_from(number_of_blocks_in_term).unwrap());
        reduced_rewards += intermediate_reward - final_block_rewards;
        pending_rewards.insert(address, final_block_rewards);
    }
    give_additional_rewards(reduced_rewards, work_info, |address, reward| {
        let prev = pending_rewards.entry(*address).or_default();
        *prev += reward;
        Ok(())
    })?;

    Ok(pending_rewards)
}

fn give_additional_rewards<F: FnMut(&Public, u64) -> Result<(), Error>>(
    mut reduced_rewards: u64,
    work_info: HashMap<Public, WorkInfo>,
    mut f: F,
) -> Result<(), Error> {
    let sorted_validators = work_info
        .into_iter()
        .map(|(address, info)| (address, Ratio::new(info.missed, info.proposed)))
        .fold(BTreeMap::<Ratio<usize>, Vec<Public>>::new(), |mut map, (address, average_missed)| {
            map.entry(average_missed).or_default().push(address);
            map
        });
    for validators in sorted_validators.values() {
        let reward = reduced_rewards / (u64::try_from(validators.len()).unwrap() + 1);
        if reward == 0 {
            break
        }
        for validator in validators {
            f(validator, reward)?;
            reduced_rewards -= reward;
        }
    }
    Ok(())
}

fn get_bitset(_seal: &[Bytes]) -> BitSet {
    unimplemented!();
}

lazy_static! {
    static ref NETWORK_ID: Mutex<Option<NetworkId>> = Mutex::new(None);
}

fn check_network_id(network_id: NetworkId) -> Result<(), Error> {
    let mut saved_network_id = NETWORK_ID.lock();
    if saved_network_id.is_none() {
        *saved_network_id = Some(network_id);
    }

    if *saved_network_id != Some(network_id) {
        return Err(Error::InvalidNetworkId(network_id))
    }

    Ok(())
}

#[allow(dead_code)]
fn create_custom_metadata() -> HashMap<&'static str, String> {
    unimplemented!();
}
