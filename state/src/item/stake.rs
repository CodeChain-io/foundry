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
use ctypes::errors::RuntimeError;
use ctypes::transaction::Validator;
use ctypes::{BlockNumber, CompactValidatorEntry, CompactValidatorSet, TransactionIndex, TransactionLocation};
use primitives::{Bytes, H256};
use rlp::{decode_list, encode_list, Decodable, Encodable, Rlp, RlpStream};
use std::cmp::{Ordering, Reverse};
use std::collections::btree_map::{BTreeMap, Entry};
use std::collections::btree_set::{self, BTreeSet};
use std::collections::{btree_map, HashMap, HashSet};
use std::ops::Deref;
use std::vec;

pub fn get_stake_account_key(pubkey: &Public) -> H256 {
    StakeKeyBuilder::new(2).append(&"Account").append(pubkey).into_key()
}

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

pub struct StakeAccount<'a> {
    pub pubkey: &'a Public,
    pub balance: StakeQuantity,
}

impl<'a> StakeAccount<'a> {
    pub fn load_from_state(state: &TopLevelState, pubkey: &'a Public) -> StateResult<StakeAccount<'a>> {
        let account_key = get_stake_account_key(pubkey);
        let action_data = state.action_data(&account_key)?;

        let balance = match action_data {
            Some(data) => Rlp::new(&data).as_val().unwrap(),
            None => StakeQuantity::default(),
        };

        Ok(StakeAccount {
            pubkey,
            balance,
        })
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let account_key = get_stake_account_key(self.pubkey);
        if self.balance != 0 {
            let rlp = rlp::encode(&self.balance);
            state.update_action_data(&account_key, rlp)?;
        } else {
            state.remove_action_data(&account_key);
        }
        Ok(())
    }

    pub fn add_balance(&mut self, amount: u64) -> Result<(), RuntimeError> {
        self.balance += amount;
        Ok(())
    }
}

pub struct Stakeholders(BTreeSet<Public>);

impl Stakeholders {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Stakeholders> {
        let action_data = state.action_data(&*STAKEHOLDER_ADDRESSES_KEY)?;
        let pubkeys = decode_set(action_data.as_ref());
        Ok(Stakeholders(pubkeys))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = *STAKEHOLDER_ADDRESSES_KEY;
        if !self.0.is_empty() {
            state.update_action_data(&key, encode_set(&self.0))?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
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

    pub fn update_by_increased_balance(&mut self, account: &StakeAccount<'_>) {
        if account.balance > 0 {
            self.0.insert(*account.pubkey);
        }
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

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = get_delegation_key(self.delegator);
        if !self.delegatees.is_empty() {
            let encoded = encode_map(&self.delegatees);
            state.update_action_data(&key, encoded)?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
    }

    pub fn add_quantity(&mut self, delegatee: Public, quantity: StakeQuantity) -> StateResult<()> {
        if quantity == 0 {
            return Ok(())
        }
        *self.delegatees.entry(delegatee).or_insert(0) += quantity;
        Ok(())
    }

    pub fn subtract_quantity(&mut self, delegatee: Public, quantity: StakeQuantity) -> StateResult<()> {
        if quantity == 0 {
            return Ok(())
        }

        if let Entry::Occupied(mut entry) = self.delegatees.entry(delegatee) {
            match entry.get().cmp(&quantity) {
                Ordering::Greater => {
                    *entry.get_mut() -= quantity;
                    return Ok(())
                }
                Ordering::Equal => {
                    entry.remove();
                    return Ok(())
                }
                Ordering::Less => {}
            }
        }

        Err(RuntimeError::FailedToHandleCustomAction("Cannot subtract more than that is delegated to".to_string())
            .into())
    }

    pub fn get_quantity(&self, delegatee: &Public) -> StakeQuantity {
        self.delegatees.get(delegatee).cloned().unwrap_or(0)
    }

    pub fn iter(&self) -> btree_map::Iter<'_, Public, StakeQuantity> {
        self.delegatees.iter()
    }
}

#[derive(Debug)]
pub struct NextValidators(Vec<Validator>);
impl NextValidators {
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

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = *CANDIDATES_KEY;
        if !self.0.is_empty() {
            let encoded = encode_iter(self.0.iter());
            state.update_action_data(&key, encoded)?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
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

    pub fn get_candidate(&self, account: &Public) -> Option<&Candidate> {
        self.0.iter().find(|c| c.pubkey == *account)
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    #[cfg(test)]
    pub fn get_index(&self, account: &Public) -> Option<usize> {
        self.0.iter().position(|c| c.pubkey == *account)
    }

    pub fn add_deposit(
        &mut self,
        pubkey: &Public,
        quantity: DepositQuantity,
        nomination_ends_at: u64,
        nomination_starts_at: TransactionLocation,
        metadata: Bytes,
    ) {
        if let Some(index) = self.0.iter().position(|c| c.pubkey == *pubkey) {
            let candidate = &mut self.0[index];
            candidate.deposit += quantity;
            if candidate.nomination_ends_at < nomination_ends_at {
                candidate.nomination_ends_at = nomination_ends_at;
            }
            candidate.metadata = metadata;
        } else {
            self.0.push(Candidate {
                pubkey: *pubkey,
                deposit: quantity,
                nomination_ends_at,
                nomination_starts_at_block_number: nomination_starts_at.block_number,
                nomination_starts_at_transaction_index: nomination_starts_at.transaction_index,
                metadata,
            });
        };
        self.reprioritize(&[*pubkey]);
    }

    pub fn renew_candidates(
        &mut self,
        validators: &[Validator],
        nomination_ends_at: u64,
        inactive_validators: &[Public],
        banned: &Banned,
    ) {
        let to_renew: HashSet<_> = (validators.iter())
            .map(|validator| *validator.pubkey())
            .filter(|pubkey| !inactive_validators.contains(pubkey))
            .collect();

        for candidate in self.0.iter_mut().filter(|c| to_renew.contains(&c.pubkey)) {
            let pubkey = candidate.pubkey;
            assert!(!banned.is_banned(&pubkey), "{:?} is banned public key", pubkey);
            candidate.nomination_ends_at = nomination_ends_at;
        }

        let to_reprioritize: Vec<_> =
            self.0.iter().filter(|c| to_renew.contains(&c.pubkey)).map(|c| c.pubkey).collect();

        self.reprioritize(&to_reprioritize);
    }

    pub fn drain_expired_candidates(&mut self, term_index: u64) -> Vec<Candidate> {
        let (expired, retained): (Vec<_>, Vec<_>) = self.0.drain(..).partition(|c| c.nomination_ends_at <= term_index);
        self.0 = retained;
        expired
    }

    pub fn remove(&mut self, pubkey: &Public) -> Option<Candidate> {
        if let Some(index) = self.0.iter().position(|c| c.pubkey == *pubkey) {
            Some(self.0.remove(index))
        } else {
            None
        }
    }

    fn reprioritize(&mut self, targets: &[Public]) {
        let mut renewed = Vec::new();
        for target in targets {
            let position =
                (self.0.iter()).position(|c| c.pubkey == *target).expect("Reprioritize target should be a candidate");
            renewed.push(self.0.remove(position));
        }
        self.0.append(&mut renewed);
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
pub enum ReleaseResult {
    NotExists,
    InCustody,
    Released(Prisoner),
}

impl Jail {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Jail> {
        let key = *JAIL_KEY;
        let prisoner = state.action_data(&key)?.map(|data| decode_list::<Prisoner>(&data)).unwrap_or_default();
        let indexed = prisoner.into_iter().map(|c| (c.pubkey, c)).collect();
        Ok(Jail(indexed))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = *JAIL_KEY;
        if !self.0.is_empty() {
            let encoded = encode_iter(self.0.values());
            state.update_action_data(&key, encoded)?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
    }

    pub fn get_prisoner(&self, pubkey: &Public) -> Option<&Prisoner> {
        self.0.get(pubkey)
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn is_empty(&self) -> bool {
        self.0.len() == 0
    }

    pub fn add(&mut self, candidate: Candidate, custody_until: u64, released_at: u64) {
        assert!(custody_until <= released_at);
        let pubkey = candidate.pubkey;
        self.0.insert(pubkey, Prisoner {
            pubkey,
            deposit: candidate.deposit,
            custody_until,
            released_at,
        });
    }

    pub fn remove(&mut self, pubkey: &Public) -> Option<Prisoner> {
        self.0.remove(pubkey)
    }

    pub fn try_release(&mut self, pubkey: &Public, term_index: u64) -> ReleaseResult {
        match self.0.entry(*pubkey) {
            Entry::Occupied(entry) => {
                if entry.get().custody_until < term_index {
                    ReleaseResult::Released(entry.remove())
                } else {
                    ReleaseResult::InCustody
                }
            }
            _ => ReleaseResult::NotExists,
        }
    }

    pub fn released_addresses(self, term_index: u64) -> Vec<Public> {
        self.0.values().filter(|c| c.released_at <= term_index).map(|c| c.pubkey).collect()
    }
}

pub struct Banned(BTreeSet<Public>);
impl Banned {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Banned> {
        let key = *BANNED_KEY;
        let action_data = state.action_data(&key)?;
        Ok(Banned(decode_set(action_data.as_ref())))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = *BANNED_KEY;
        if !self.0.is_empty() {
            let encoded = encode_set(&self.0);
            state.update_action_data(&key, encoded)?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
    }

    pub fn add(&mut self, pubkey: Public) {
        self.0.insert(pubkey);
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

fn encode_set<V>(set: &BTreeSet<V>) -> Vec<u8>
where
    V: Ord + Encodable, {
    let mut rlp = RlpStream::new();
    rlp.begin_list(set.len());
    for value in set.iter() {
        rlp.append(value);
    }
    rlp.drain()
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

fn encode_map<K, V>(map: &BTreeMap<K, V>) -> Vec<u8>
where
    K: Ord + Encodable,
    V: Encodable, {
    let mut rlp = RlpStream::new();
    encode_map_impl(&mut rlp, map);
    rlp.drain()
}

fn encode_map_impl<K, V>(rlp: &mut RlpStream, map: &BTreeMap<K, V>)
where
    K: Ord + Encodable,
    V: Encodable, {
    rlp.begin_list(map.len());
    for (key, value) in map.iter() {
        let record = rlp.begin_list(2);
        record.append(key);
        record.append(value);
    }
}

fn encode_iter<'a, V, I>(iter: I) -> Vec<u8>
where
    V: 'a + Encodable,
    I: ExactSizeIterator<Item = &'a V> + Clone, {
    let mut rlp = RlpStream::new();
    rlp.begin_list(iter.clone().count());
    for value in iter {
        rlp.append(value);
    }
    rlp.drain()
}

#[allow(clippy::implicit_hasher)] // XXX: Fix this clippy warning if it becomes a real problem.
#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::helpers;
    use rand::{Rng, SeedableRng};
    use rand_xorshift::XorShiftRng;

    fn rng() -> XorShiftRng {
        let seed: [u8; 16] = [0, 1, 2, 3, 4, 5, 6, 7, 0, 1, 2, 3, 4, 5, 6, 7];
        XorShiftRng::from_seed(seed)
    }

    #[test]
    fn default_balance_is_zero() {
        let state = helpers::get_temp_state();
        let pubkey = Public::random();
        let account = StakeAccount::load_from_state(&state, &pubkey).unwrap();
        assert_eq!(account.pubkey, &pubkey);
        assert_eq!(account.balance, 0);
    }

    #[test]
    fn balance_add() {
        let mut state = helpers::get_temp_state();
        let pubkey = Public::random();
        {
            let mut account = StakeAccount::load_from_state(&state, &pubkey).unwrap();
            account.add_balance(100).unwrap();
            account.save_to_state(&mut state).unwrap();
        }
        let account = StakeAccount::load_from_state(&state, &pubkey).unwrap();
        assert_eq!(account.balance, 100);
    }

    #[test]
    fn stakeholders_track() {
        let mut rng = rng();
        let mut state = helpers::get_temp_state();
        let pubkeys: Vec<_> = (1..100).map(Public::from).collect();
        let accounts: Vec<_> = pubkeys
            .iter()
            .map(|pubkey| StakeAccount {
                pubkey,
                balance: rng.gen_range(1, 100),
            })
            .collect();

        let mut stakeholders = Stakeholders::load_from_state(&state).unwrap();
        for account in &accounts {
            stakeholders.update_by_increased_balance(account);
        }
        stakeholders.save_to_state(&mut state).unwrap();

        let stakeholders = Stakeholders::load_from_state(&state).unwrap();
        assert!(pubkeys.iter().all(|pubkey| stakeholders.contains(pubkey)));
    }

    #[test]
    fn initial_delegation_is_empty() {
        let state = helpers::get_temp_state();

        let delegatee = Public::random();
        let delegation = Delegation::load_from_state(&state, &delegatee).unwrap();
        assert_eq!(delegation.delegator, &delegatee);
        assert_eq!(delegation.iter().count(), 0);
    }

    #[test]
    fn delegation_add() {
        let mut rng = rng();
        let mut state = helpers::get_temp_state();

        // Prepare
        let delegator = Public::random();
        let delegatees: Vec<_> = (0..10).map(Public::from).collect();
        let delegation_amount: HashMap<&Public, StakeQuantity> =
            delegatees.iter().map(|pubkey| (pubkey, rng.gen_range(0, 100))).collect();

        // Do delegate
        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        for delegatee in delegatees.iter() {
            delegation.add_quantity(*delegatee, delegation_amount[delegatee]).unwrap()
        }
        delegation.save_to_state(&mut state).unwrap();

        // assert
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.iter().count(), delegatees.len());
        for delegatee in delegatees.iter() {
            assert_eq!(delegation.get_quantity(delegatee), delegation_amount[delegatee]);
        }
    }

    #[test]
    fn delegation_zero_add_should_not_be_included() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let delegator = Public::random();
        let delegatee1 = Public::random();
        let delegatee2 = Public::random();

        // Do delegate
        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        delegation.add_quantity(delegatee1, 100).unwrap();
        delegation.add_quantity(delegatee2, 0).unwrap();
        delegation.save_to_state(&mut state).unwrap();

        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        let delegated = delegation.iter().collect::<Vec<_>>();
        assert_eq!(&delegated, &[(&delegatee1, &100)]);
    }

    #[test]
    fn delegation_can_subtract() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let delegator = Public::random();
        let delegatee = Public::random();

        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        delegation.add_quantity(delegatee, 100).unwrap();
        delegation.save_to_state(&mut state).unwrap();

        // Do subtract
        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        delegation.subtract_quantity(delegatee, 30).unwrap();
        delegation.save_to_state(&mut state).unwrap();

        // Assert
        let delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert_eq!(delegation.get_quantity(&delegatee), 70);
    }

    #[test]
    fn delegation_cannot_subtract_mor_than_delegated() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let delegator = Public::random();
        let delegatee = Public::random();

        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        delegation.add_quantity(delegatee, 100).unwrap();
        delegation.save_to_state(&mut state).unwrap();

        // Do subtract
        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        assert!(delegation.subtract_quantity(delegatee, 130).is_err());
    }

    #[test]
    fn delegation_empty_removed_from_state() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let delegator = Public::random();
        let delegatee = Public::random();

        // Do delegate
        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        delegation.add_quantity(delegatee, 0).unwrap();
        delegation.save_to_state(&mut state).unwrap();

        let result = state.action_data(&get_delegation_key(&delegator)).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn delegation_became_empty_removed_from_state() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let delegator = Public::random();
        let delegatee = Public::random();

        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        delegation.add_quantity(delegatee, 100).unwrap();
        delegation.save_to_state(&mut state).unwrap();

        // Do subtract
        let mut delegation = Delegation::load_from_state(&state, &delegator).unwrap();
        delegation.subtract_quantity(delegatee, 100).unwrap();
        delegation.save_to_state(&mut state).unwrap();

        // Assert
        let result = state.action_data(&get_delegation_key(&delegator)).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn candidates_deposit_add() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = Public::random();
        let deposits = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10];

        let nomination_ends_at = 0;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        for deposit in deposits.iter() {
            let mut candidates = Candidates::load_from_state(&state).unwrap();
            candidates.add_deposit(&pubkey, *deposit, nomination_ends_at, nomination_starts_at, b"".to_vec());
            candidates.save_to_state(&mut state).unwrap();
        }

        // Assert
        let candidates = Candidates::load_from_state(&state).unwrap();
        let candidate = candidates.get_candidate(&pubkey);
        assert_ne!(candidate, None);
        assert_eq!(candidate.unwrap().deposit, 55);
    }

    #[test]
    fn candidates_metadata() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = Public::random();

        let mut candidates = Candidates::load_from_state(&state).unwrap();
        let deposit = 10;
        let nomination_ends_at = 0;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        candidates.add_deposit(&pubkey, deposit, nomination_ends_at, nomination_starts_at, b"metadata".to_vec());
        candidates.save_to_state(&mut state).unwrap();

        // Assert
        let candidates = Candidates::load_from_state(&state).unwrap();
        let candidate = candidates.get_candidate(&pubkey);
        assert_ne!(candidate, None);
        assert_eq!(&candidate.unwrap().metadata, b"metadata");
    }

    #[test]
    fn candidates_update_metadata() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = Public::random();

        let mut candidates = Candidates::load_from_state(&state).unwrap();
        let nomination_ends_at = 0;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        candidates.add_deposit(&pubkey, 10, nomination_ends_at, nomination_starts_at, b"metadata".to_vec());
        candidates.save_to_state(&mut state).unwrap();

        let mut candidates = Candidates::load_from_state(&state).unwrap();
        candidates.add_deposit(&pubkey, 10, nomination_ends_at, nomination_starts_at, b"metadata-updated".to_vec());
        candidates.save_to_state(&mut state).unwrap();

        // Assert
        let candidates = Candidates::load_from_state(&state).unwrap();
        let candidate = candidates.get_candidate(&pubkey);
        assert_ne!(candidate, None);
        assert_eq!(&candidate.unwrap().metadata, b"metadata-updated");
    }

    #[test]
    fn candidates_deposit_can_be_zero() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = Public::random();
        let mut candidates = Candidates::load_from_state(&state).unwrap();
        let nomination_ends_at = 10;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        candidates.add_deposit(&pubkey, 0, nomination_ends_at, nomination_starts_at, b"".to_vec());
        candidates.save_to_state(&mut state).unwrap();

        // Assert
        let candidates = Candidates::load_from_state(&state).unwrap();
        let candidate = candidates.get_candidate(&pubkey);
        assert_ne!(candidate, None);
        assert_eq!(candidate.unwrap().deposit, 0);
        assert_eq!(candidate.unwrap().nomination_ends_at, 10, "Can be a candidate with 0 deposit");
    }

    #[test]
    fn candidates_update_metadata_with_zero_additional_deposit() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = Public::random();

        let mut candidates = Candidates::load_from_state(&state).unwrap();
        let nomination_ends_at = 0;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        candidates.add_deposit(&pubkey, 10, nomination_ends_at, nomination_starts_at, b"metadata".to_vec());
        candidates.save_to_state(&mut state).unwrap();

        let mut candidates = Candidates::load_from_state(&state).unwrap();
        candidates.add_deposit(&pubkey, 0, nomination_ends_at, nomination_starts_at, b"metadata-updated".to_vec());
        candidates.save_to_state(&mut state).unwrap();

        // Assert
        let candidates = Candidates::load_from_state(&state).unwrap();
        let candidate = candidates.get_candidate(&pubkey);
        assert_ne!(candidate, None);
        assert_eq!(&candidate.unwrap().metadata, b"metadata-updated");
    }

    #[test]
    fn candidates_deposit_should_update_nomination_ends_at() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = Public::random();
        let deposit_and_nomination_ends_at = [(10, 11), (20, 22), (30, 33), (0, 44)];

        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        for (deposit, nomination_ends_at) in &deposit_and_nomination_ends_at {
            let mut candidates = Candidates::load_from_state(&state).unwrap();
            candidates.add_deposit(&pubkey, *deposit, *nomination_ends_at, nomination_starts_at, b"".to_vec());
            candidates.save_to_state(&mut state).unwrap();
        }

        // Assert
        let candidates = Candidates::load_from_state(&state).unwrap();
        let candidate = candidates.get_candidate(&pubkey);
        assert_ne!(candidate, None);
        assert_eq!(candidate.unwrap().deposit, 60);
        assert_eq!(
            candidate.unwrap().nomination_ends_at,
            44,
            "nomination_ends_at should be updated incrementally, and including zero deposit"
        );
    }

    #[test]
    fn candidates_can_remove_expired_deposit() {
        let mut state = helpers::get_temp_state();

        let pubkey0 = 0.into();
        let pubkey1 = 1.into();
        let pubkey2 = 2.into();
        let pubkey3 = 3.into();

        // Prepare
        let candidates_prepared = [
            Candidate {
                pubkey: pubkey0,
                deposit: 20,
                nomination_ends_at: 11,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            Candidate {
                pubkey: pubkey1,
                deposit: 30,
                nomination_ends_at: 22,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            Candidate {
                pubkey: pubkey2,
                deposit: 40,
                nomination_ends_at: 33,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            Candidate {
                pubkey: pubkey3,
                deposit: 50,
                nomination_ends_at: 44,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
        ];

        for Candidate {
            pubkey,
            deposit,
            nomination_ends_at,
            nomination_starts_at_block_number,
            nomination_starts_at_transaction_index,
            metadata,
        } in &candidates_prepared
        {
            let mut candidates = Candidates::load_from_state(&state).unwrap();
            candidates.add_deposit(
                &pubkey,
                *deposit,
                *nomination_ends_at,
                TransactionLocation {
                    block_number: *nomination_starts_at_block_number,
                    transaction_index: *nomination_starts_at_transaction_index,
                },
                metadata.clone(),
            );
            candidates.save_to_state(&mut state).unwrap();
        }

        // Remove Expired
        let mut candidates = Candidates::load_from_state(&state).unwrap();
        let mut expired = candidates.drain_expired_candidates(22);
        candidates.save_to_state(&mut state).unwrap();

        // Assert
        expired.sort_unstable_by_key(|c| c.pubkey);
        let mut prepared_expired = candidates_prepared[0..=1].to_vec();
        prepared_expired.sort_unstable_by_key(|c| c.pubkey);
        assert_eq!(expired, prepared_expired);
        let candidates = Candidates::load_from_state(&state).unwrap();
        assert_eq!(candidates.len(), 2);
        assert_eq!(candidates.get_candidate(&candidates_prepared[2].pubkey), Some(&candidates_prepared[2]));
        assert_eq!(candidates.get_candidate(&candidates_prepared[3].pubkey), Some(&candidates_prepared[3]));
    }

    #[test]
    fn candidates_expire_all_cleanup_state() {
        let mut state = helpers::get_temp_state();

        let pubkey0 = 0.into();
        let pubkey1 = 1.into();
        let pubkey2 = 2.into();
        let pubkey3 = 3.into();

        // Prepare
        let candidates_prepared = [
            Candidate {
                pubkey: pubkey0,
                deposit: 20,
                nomination_ends_at: 11,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            Candidate {
                pubkey: pubkey1,
                deposit: 30,
                nomination_ends_at: 22,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            Candidate {
                pubkey: pubkey2,
                deposit: 40,
                nomination_ends_at: 33,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            Candidate {
                pubkey: pubkey3,
                deposit: 50,
                nomination_ends_at: 44,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
        ];

        for Candidate {
            pubkey,
            deposit,
            nomination_ends_at,
            nomination_starts_at_block_number,
            nomination_starts_at_transaction_index,
            metadata,
        } in &candidates_prepared
        {
            let mut candidates = Candidates::load_from_state(&state).unwrap();
            candidates.add_deposit(
                &pubkey,
                *deposit,
                *nomination_ends_at,
                TransactionLocation {
                    block_number: *nomination_starts_at_block_number,
                    transaction_index: *nomination_starts_at_transaction_index,
                },
                metadata.clone(),
            );
            candidates.save_to_state(&mut state).unwrap();
        }

        // Remove Expired
        let mut candidates = Candidates::load_from_state(&state).unwrap();
        let mut expired = candidates.drain_expired_candidates(99);
        candidates.save_to_state(&mut state).unwrap();

        expired.sort_unstable_by_key(|c| c.pubkey);
        let mut prepared_expired = candidates_prepared[0..4].to_vec();
        prepared_expired.sort_unstable_by_key(|c| c.pubkey);

        // Assert
        assert_eq!(expired, prepared_expired);
        let result = state.action_data(&*CANDIDATES_KEY).unwrap();
        assert_eq!(result, None);
    }

    #[test]
    fn jail_try_free_not_existing() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = 1.into();
        let mut jail = Jail::load_from_state(&state).unwrap();
        jail.add(
            Candidate {
                pubkey,
                deposit: 100,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            10,
            20,
        );
        jail.save_to_state(&mut state).unwrap();

        let mut jail = Jail::load_from_state(&state).unwrap();
        let freed = jail.try_release(&Public::from(1000), 5);
        assert_eq!(freed, ReleaseResult::NotExists);
        assert_eq!(jail.len(), 1);
        assert_ne!(jail.get_prisoner(&pubkey), None);
    }

    #[test]
    fn jail_try_release_none_until_custody() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = 1.into();
        let mut jail = Jail::load_from_state(&state).unwrap();
        jail.add(
            Candidate {
                pubkey,
                deposit: 100,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            10,
            20,
        );
        jail.save_to_state(&mut state).unwrap();

        let mut jail = Jail::load_from_state(&state).unwrap();
        let released = jail.try_release(&pubkey, 10);
        assert_eq!(released, ReleaseResult::InCustody);
        assert_eq!(jail.len(), 1);
        assert_ne!(jail.get_prisoner(&pubkey), None);
    }

    #[test]
    fn jail_try_release_prisoner_after_custody() {
        let mut state = helpers::get_temp_state();

        // Prepare
        let pubkey = 1.into();
        let mut jail = Jail::load_from_state(&state).unwrap();
        jail.add(
            Candidate {
                pubkey,
                deposit: 100,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            10,
            20,
        );
        jail.save_to_state(&mut state).unwrap();

        let mut jail = Jail::load_from_state(&state).unwrap();
        let released = jail.try_release(&pubkey, 11);
        jail.save_to_state(&mut state).unwrap();

        // Assert
        assert_eq!(
            released,
            ReleaseResult::Released(Prisoner {
                pubkey,
                deposit: 100,
                custody_until: 10,
                released_at: 20,
            })
        );
        assert_eq!(jail.len(), 0);
        assert_eq!(jail.get_prisoner(&pubkey), None);

        let result = state.action_data(&*JAIL_KEY).unwrap();
        assert_eq!(result, None, "Should clean the state if all prisoners are released");
    }

    #[test]
    fn jail_keep_prisoners_until_kick_at() {
        let mut state = helpers::get_temp_state();

        let pubkey1 = 1.into();
        let pubkey2 = 2.into();

        // Prepare
        let mut jail = Jail::load_from_state(&state).unwrap();
        jail.add(
            Candidate {
                pubkey: pubkey1,
                deposit: 100,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            10,
            20,
        );
        jail.add(
            Candidate {
                pubkey: pubkey2,
                deposit: 200,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            15,
            25,
        );
        jail.save_to_state(&mut state).unwrap();

        // Kick
        let mut jail = Jail::load_from_state(&state).unwrap();
        let released =
            jail.clone().released_addresses(19).iter().map(|address| jail.remove(address).unwrap()).collect::<Vec<_>>();
        jail.save_to_state(&mut state).unwrap();

        // Assert
        assert_eq!(released, Vec::new());
        assert_eq!(jail.len(), 2);
        assert_ne!(jail.get_prisoner(&pubkey1), None);
        assert_ne!(jail.get_prisoner(&pubkey2), None);
    }

    #[test]
    fn jail_partially_kick_prisoners() {
        let mut state = helpers::get_temp_state();

        let pubkey1 = 1.into();
        let pubkey2 = 2.into();

        // Prepare
        let mut jail = Jail::load_from_state(&state).unwrap();
        jail.add(
            Candidate {
                pubkey: pubkey1,
                deposit: 100,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            10,
            20,
        );
        jail.add(
            Candidate {
                pubkey: pubkey2,
                deposit: 200,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            15,
            25,
        );
        jail.save_to_state(&mut state).unwrap();

        // Kick
        let mut jail = Jail::load_from_state(&state).unwrap();
        let released = jail
            .clone()
            .released_addresses(20)
            .into_iter()
            .map(|address| jail.remove(&address).unwrap())
            .collect::<Vec<_>>();
        jail.save_to_state(&mut state).unwrap();

        // Assert
        assert_eq!(released, vec![Prisoner {
            pubkey: pubkey1,
            deposit: 100,
            custody_until: 10,
            released_at: 20,
        }]);
        assert_eq!(jail.len(), 1);
        assert_eq!(jail.get_prisoner(&pubkey1), None);
        assert_ne!(jail.get_prisoner(&pubkey2), None);
    }

    #[test]
    fn jail_kick_all_prisoners() {
        let mut state = helpers::get_temp_state();

        let pubkey1 = 1.into();
        let pubkey2 = 2.into();

        // Prepare
        let mut jail = Jail::load_from_state(&state).unwrap();
        jail.add(
            Candidate {
                pubkey: pubkey1,
                deposit: 100,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            10,
            20,
        );
        jail.add(
            Candidate {
                pubkey: pubkey2,
                deposit: 200,
                nomination_ends_at: 0,
                nomination_starts_at_block_number: 0,
                nomination_starts_at_transaction_index: 0,
                metadata: b"".to_vec(),
            },
            15,
            25,
        );
        jail.save_to_state(&mut state).unwrap();

        // Kick
        let mut jail = Jail::load_from_state(&state).unwrap();
        let released = jail
            .clone()
            .released_addresses(25)
            .into_iter()
            .map(|address| jail.remove(&address).unwrap())
            .collect::<Vec<_>>();
        jail.save_to_state(&mut state).unwrap();

        // Assert
        assert_eq!(released, vec![
            Prisoner {
                pubkey: pubkey1,
                deposit: 100,
                custody_until: 10,
                released_at: 20,
            },
            Prisoner {
                pubkey: pubkey2,
                deposit: 200,
                custody_until: 15,
                released_at: 25,
            }
        ]);
        assert_eq!(jail.len(), 0);
        assert_eq!(jail.get_prisoner(&pubkey1), None);
        assert_eq!(jail.get_prisoner(&pubkey2), None);

        let result = state.action_data(&*JAIL_KEY).unwrap();
        assert_eq!(result, None, "Should clean the state if all prisoners are released");
    }

    #[test]
    fn empty_ban_save_clean_state() {
        let mut state = helpers::get_temp_state();
        let banned = Banned::load_from_state(&state).unwrap();
        banned.save_to_state(&mut state).unwrap();

        let result = state.action_data(&*BANNED_KEY).unwrap();
        assert_eq!(result, None, "Should clean the state if there are no banned accounts");
    }

    #[test]
    fn added_to_ban_is_banned() {
        let mut state = helpers::get_temp_state();

        let pubkey = Public::from(1);
        let innocent = Public::from(2);

        let mut banned = Banned::load_from_state(&state).unwrap();
        banned.add(pubkey);
        banned.save_to_state(&mut state).unwrap();

        let banned = Banned::load_from_state(&state).unwrap();
        assert!(banned.is_banned(&pubkey));
        assert!(!banned.is_banned(&innocent));
    }

    #[test]
    fn latest_deposit_higher_priority() {
        let mut state = helpers::get_temp_state();
        let pubkeys = (0..10).map(|_| Public::random()).collect::<Vec<_>>();

        let mut candidates = Candidates::load_from_state(&state).unwrap();
        let nomination_ends_at = 0;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        for _ in 0..10 {
            // Random pre-fill
            let i = rand::thread_rng().gen_range(0, pubkeys.len());
            let pubkey = &pubkeys[i];
            candidates.add_deposit(pubkey, 0, nomination_ends_at, nomination_starts_at, Bytes::new());
        }
        // Inserting pubkey in this order, they'll get sorted.
        for pubkey in &pubkeys {
            candidates.add_deposit(pubkey, 10, nomination_ends_at, nomination_starts_at, Bytes::new());
        }
        candidates.save_to_state(&mut state).unwrap();

        let candidates = Candidates::load_from_state(&state).unwrap();
        let results: Vec<_> = pubkeys.iter().map(|pubkey| candidates.get_index(&pubkey)).collect();
        // TODO assert!(results.is_sorted(), "Should be sorted in the insertion order");
        for i in 0..results.len() - 1 {
            assert!(results[i] < results[i + 1], "Should be sorted in the insertion order");
        }
    }

    #[test]
    fn renew_doesnt_change_relative_priority() {
        let mut state = helpers::get_temp_state();
        let pubkeys = (0..10).map(|_| Public::random()).collect::<Vec<_>>();

        let mut candidates = Candidates::load_from_state(&state).unwrap();
        let nomination_ends_at = 0;
        let nomination_starts_at = TransactionLocation {
            block_number: 0,
            transaction_index: 0,
        };
        for pubkey in &pubkeys {
            candidates.add_deposit(pubkey, 10, nomination_ends_at, nomination_starts_at, Bytes::new());
        }
        candidates.save_to_state(&mut state).unwrap();

        let dummy_validators =
            pubkeys[0..5].iter().map(|pubkey| Validator::new(0, 0, *pubkey, 0, 0)).collect::<Vec<_>>();
        let dummy_banned = Banned::load_from_state(&state).unwrap();
        candidates.renew_candidates(&dummy_validators, 0, &[], &dummy_banned);

        let indexes: Vec<_> = pubkeys.iter().map(|pubkey| candidates.get_index(pubkey).unwrap()).collect();
        assert_eq!(indexes, vec![5, 6, 7, 8, 9, 0, 1, 2, 3, 4]);
    }
}
