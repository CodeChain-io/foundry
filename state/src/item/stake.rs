// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::{ActionData, StakeKeyBuilder, StateResult, TopLevelState, TopState, TopStateView};
use ckey::Ed25519Public as Public;
use ctypes::transaction::Validator;
use ctypes::{BlockNumber, CompactValidatorEntry, CompactValidatorSet, TransactionIndex};
use primitives::{Bytes, H256};
use rlp::{decode_list, encode_list, Decodable, Rlp};
use std::cmp::Reverse;
use std::collections::btree_map::BTreeMap;
use std::collections::btree_set::{self, BTreeSet};
use std::collections::{btree_map, HashMap};
use std::ops::Deref;
use std::vec;

lazy_static! {
    pub static ref STAKEHOLDER_ADDRESSES_KEY: H256 = StakeKeyBuilder::new(1).append(&"StakeholderAddresses").into_key();
    pub static ref CANDIDATES_KEY: H256 = StakeKeyBuilder::new(1).append(&"Candidates").into_key();
    pub static ref JAIL_KEY: H256 = StakeKeyBuilder::new(1).append(&"Jail").into_key();
    pub static ref BANNED_KEY: H256 = StakeKeyBuilder::new(1).append(&"Banned").into_key();
    pub static ref NEXT_VALIDATORS_KEY: H256 = StakeKeyBuilder::new(1).append(&"Validators").into_key();
    pub static ref CURRENT_VALIDATORS_KEY: H256 = StakeKeyBuilder::new(1).append(&"CurrentValidators").into_key();
}

pub fn get_delegation_key(pubkey: &Public) -> H256 {
    StakeKeyBuilder::new(2).append(&"Delegation").append(pubkey).into_key()
}

pub type StakeQuantity = u64;
pub type DepositQuantity = u64;

pub struct Stakeholders(BTreeSet<Public>);

impl Stakeholders {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Stakeholders> {
        let action_data = state.action_data(&*STAKEHOLDER_ADDRESSES_KEY)?;
        let pubkeys = decode_set(action_data.as_ref());
        Ok(Stakeholders(pubkeys))
    }

    fn delegatees(state: &TopLevelState) -> StateResult<HashMap<Public, u64>> {
        let stakeholders = Stakeholders::load_from_state(state)?;
        let mut result = HashMap::new();
        for stakeholder in stakeholders.iter() {
            let delegation = Delegation::load_from_state(state, stakeholder)?;
            for (delegatee, quantity) in delegation.iter() {
                *result.entry(*delegatee).or_default() += *quantity;
            }
        }
        Ok(result)
    }

    #[allow(dead_code)]
    pub fn contains(&self, pubkey: &Public) -> bool {
        self.0.contains(pubkey)
    }
    pub fn iter(&self) -> btree_set::Iter<'_, Public> {
        self.0.iter()
    }
}

pub struct Delegation<'a> {
    pub delegator: &'a Public,
    delegatees: BTreeMap<Public, StakeQuantity>,
}

impl<'a> Delegation<'a> {
    pub fn load_from_state(state: &TopLevelState, delegator: &'a Public) -> StateResult<Delegation<'a>> {
        let key = get_delegation_key(delegator);
        let action_data = state.action_data(&key)?;
        let delegatees = decode_map(action_data.as_ref());

        Ok(Delegation {
            delegator,
            delegatees,
        })
    }

    pub fn iter(&self) -> btree_map::Iter<'_, Public, StakeQuantity> {
        self.delegatees.iter()
    }
}

#[derive(Debug)]
pub struct NextValidators(Vec<Validator>);
impl NextValidators {
    pub fn from_vector(vec: Vec<Validator>) -> Self {
        Self(vec)
    }

    pub fn load_from_state(state: &TopLevelState) -> StateResult<Self> {
        let key = &*NEXT_VALIDATORS_KEY;
        let validators = state.action_data(&key)?.map(|data| decode_list(&data)).unwrap_or_default();

        Ok(Self(validators))
    }

    pub fn create_compact_validator_set(&self) -> CompactValidatorSet {
        CompactValidatorSet::new(
            self.0
                .iter()
                .map(|x| CompactValidatorEntry {
                    public_key: *x.pubkey(),
                    delegation: x.delegation(),
                })
                .collect(),
        )
    }

    pub fn elect(state: &TopLevelState) -> StateResult<Self> {
        let (delegation_threshold, max_num_of_validators, min_num_of_validators, min_deposit) = {
            let metadata = state.metadata()?.expect("Metadata must exist");
            let common_params = metadata.params();
            (
                common_params.delegation_threshold(),
                common_params.max_num_of_validators(),
                common_params.min_num_of_validators(),
                common_params.min_deposit(),
            )
        };
        assert!(max_num_of_validators >= min_num_of_validators);

        let delegatees = Stakeholders::delegatees(&state)?;
        // Step 1 & 2.
        // Ordered by (delegation DESC, deposit DESC, nomination_starts_at ASC)
        let mut validators = Candidates::prepare_validators(&state, min_deposit, &delegatees)?;

        let banned = Banned::load_from_state(&state)?;
        for validator in &validators {
            assert!(!banned.is_banned(validator.pubkey()), "{:?} is banned public key", validator.pubkey());
        }

        // Step 3
        validators.truncate(max_num_of_validators);

        if validators.len() < min_num_of_validators {
            cerror!(
                ENGINE,
                "There must be something wrong. {}, {} < {}",
                "validators.len() < min_num_of_validators",
                validators.len(),
                min_num_of_validators
            );
        }
        // Step 4 & 5
        let (minimum, rest) = validators.split_at(min_num_of_validators.min(validators.len()));
        let over_threshold = rest.iter().filter(|c| c.delegation() >= delegation_threshold);

        let mut result: Vec<_> = minimum.iter().chain(over_threshold).cloned().collect();
        result.sort_unstable_by_key(|v| *v.pubkey());

        Ok(Self(result))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = &*NEXT_VALIDATORS_KEY;
        if !self.is_empty() {
            state.update_action_data(&key, encode_list(&self.0).to_vec())?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
    }

    pub fn update_weight(state: &TopLevelState, block_author: &Public) -> StateResult<Self> {
        let mut validators = Self::load_from_state(state)?;
        let min_delegation = validators.min_delegation();
        let mut sorted_validators_view: Vec<&mut Validator> = validators.0.iter_mut().collect();
        sorted_validators_view.sort_unstable_by_key(|val| {
            (
                Reverse(val.weight()),
                Reverse(val.deposit()),
                val.nominated_at_block_number(),
                val.nominated_at_transaction_index(),
            )
        });
        for validator in sorted_validators_view.iter_mut() {
            if *validator.pubkey() == *block_author {
                // block author
                validator.set_weight(validator.weight().saturating_sub(min_delegation));
                break
            }
            // neglecting validators
            validator.set_weight(validator.weight().saturating_sub(min_delegation * 2));
        }
        if validators.0.iter().all(|validator| validator.weight() == 0) {
            validators.0.iter_mut().for_each(Validator::reset);
        }
        Ok(validators)
    }

    pub fn remove(&mut self, target: &Public) {
        self.0.retain(|v| *target != *v.pubkey());
    }

    pub fn delegation(&self, pubkey: &Public) -> Option<StakeQuantity> {
        self.0.iter().find(|validator| *validator.pubkey() == *pubkey).map(|&validator| validator.delegation())
    }

    fn min_delegation(&self) -> StakeQuantity {
        self.0.iter().map(|&validator| validator.delegation()).min().expect("There must be at least one validators")
    }
}

impl Deref for NextValidators {
    type Target = Vec<Validator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Vec<Validator>> for NextValidators {
    fn from(validators: Vec<Validator>) -> Self {
        Self(validators)
    }
}

impl From<NextValidators> for Vec<Validator> {
    fn from(val: NextValidators) -> Self {
        val.0
    }
}

impl IntoIterator for NextValidators {
    type Item = Validator;
    type IntoIter = vec::IntoIter<Self::Item>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[derive(Debug)]
pub struct CurrentValidators(Vec<Validator>);
impl CurrentValidators {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Self> {
        let key = &*CURRENT_VALIDATORS_KEY;
        let validators: Vec<Validator> = state.action_data(&key)?.map(|data| decode_list(&data)).unwrap_or_default();

        Ok(Self(validators))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = &*CURRENT_VALIDATORS_KEY;
        if !self.is_empty() {
            state.update_action_data(&key, encode_list(&self.0).to_vec())?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
    }

    /// validator should be sorted by public key
    pub fn update(&mut self, validators: Vec<Validator>) {
        debug_assert_eq!(
            validators,
            {
                let mut cloned = validators.clone();
                cloned.sort_unstable_by_key(|v| *v.pubkey());
                cloned
            },
            "CurrentValidator is always sorted by public key"
        );
        self.0 = validators;
    }

    pub fn pubkeys(&self) -> Vec<Public> {
        self.0.iter().rev().map(|v| *v.pubkey()).collect()
    }

    pub fn create_compact_validator_set(&self) -> CompactValidatorSet {
        CompactValidatorSet::new(
            self.0
                .iter()
                .map(|x| CompactValidatorEntry {
                    public_key: *x.pubkey(),
                    delegation: x.delegation(),
                })
                .collect(),
        )
    }
}

impl Deref for CurrentValidators {
    type Target = Vec<Validator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<CurrentValidators> for Vec<Validator> {
    fn from(val: CurrentValidators) -> Self {
        val.0
    }
}

#[derive(Default)]
pub struct Candidates(Vec<Candidate>);
#[derive(Clone, Debug, Eq, PartialEq, RlpEncodable, RlpDecodable)]
pub struct Candidate {
    pub pubkey: Public,
    pub deposit: DepositQuantity,
    pub nomination_ends_at: u64,
    pub nomination_starts_at_block_number: BlockNumber,
    pub nomination_starts_at_transaction_index: TransactionIndex,
    pub metadata: Bytes,
}

impl Candidates {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Candidates> {
        let key = *CANDIDATES_KEY;
        let candidates = state.action_data(&key)?.map(|data| decode_list::<Candidate>(&data)).unwrap_or_default();
        Ok(Candidates(candidates))
    }

    // Sorted list of validators in ascending order of (delegation, deposit, priority).
    fn prepare_validators(
        state: &TopLevelState,
        min_deposit: DepositQuantity,
        delegations: &HashMap<Public, StakeQuantity>,
    ) -> StateResult<Vec<Validator>> {
        let Candidates(candidates) = Self::load_from_state(state)?;
        let mut result = Vec::new();
        for candidate in candidates.into_iter().filter(|c| c.deposit >= min_deposit) {
            if let Some(delegation) = delegations.get(&candidate.pubkey).cloned() {
                result.push(Validator::new(
                    delegation,
                    candidate.deposit,
                    candidate.pubkey,
                    candidate.nomination_starts_at_block_number,
                    candidate.nomination_starts_at_transaction_index,
                ));
            }
        }
        // Descending order of (delegation, deposit, priority)
        result.sort_unstable_by_key(|v| {
            (
                Reverse(v.delegation()),
                Reverse(v.deposit()),
                v.nominated_at_block_number(),
                v.nominated_at_transaction_index(),
            )
        });
        Ok(result)
    }
}

#[derive(Clone)]
pub struct Jail(BTreeMap<Public, Prisoner>);
#[derive(Clone, Debug, Eq, PartialEq, RlpEncodable, RlpDecodable)]
pub struct Prisoner {
    pub pubkey: Public,
    pub deposit: DepositQuantity,
    pub custody_until: u64,
    pub released_at: u64,
}

#[derive(Debug, Eq, PartialEq)]
pub enum ReleaseResult {}

pub struct Banned(BTreeSet<Public>);
impl Banned {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Banned> {
        let key = *BANNED_KEY;
        let action_data = state.action_data(&key)?;
        Ok(Banned(decode_set(action_data.as_ref())))
    }

    pub fn is_banned(&self, pubkey: &Public) -> bool {
        self.0.contains(pubkey)
    }
}

fn decode_set<V>(data: Option<&ActionData>) -> BTreeSet<V>
where
    V: Ord + Decodable, {
    let mut result = BTreeSet::new();
    if let Some(rlp) = data.map(|x| Rlp::new(x)) {
        for record in rlp.iter() {
            let value: V = record.as_val().unwrap();
            result.insert(value);
        }
    }
    result
}

fn decode_map<K, V>(data: Option<&ActionData>) -> BTreeMap<K, V>
where
    K: Ord + Decodable,
    V: Decodable, {
    if let Some(rlp) = data.map(|x| Rlp::new(x)) {
        decode_map_impl(rlp)
    } else {
        Default::default()
    }
}

fn decode_map_impl<K, V>(rlp: Rlp<'_>) -> BTreeMap<K, V>
where
    K: Ord + Decodable,
    V: Decodable, {
    let mut result = BTreeMap::new();
    for record in rlp.iter() {
        let key: K = record.val_at(0).unwrap();
        let value: V = record.val_at(1).unwrap();
        assert_eq!(Ok(2), record.item_count());
        result.insert(key, value);
    }
    result
}

#[allow(clippy::implicit_hasher)] // XXX: Fix this clippy warning if it becomes a real problem.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers;

    #[test]
    fn initial_delegation_is_empty() {
        let state = helpers::get_temp_state();

        let delegatee = Public::random();
        let delegation = Delegation::load_from_state(&state, &delegatee).unwrap();
        assert_eq!(delegation.delegator, &delegatee);
        assert_eq!(delegation.iter().count(), 0);
    }
}
