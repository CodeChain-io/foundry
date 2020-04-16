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

use crate::error::{Insufficient, Mismatch};
use crate::runtime_error::Error;
use crate::types::*;
use crate::{deserialize, serialize, substorage};
use serde::{de::DeserializeOwned, ser::Serialize};
use std::cmp::{max, Ordering};
use std::collections::{
    btree_map::{self, Entry},
    btree_set, BTreeMap, BTreeSet, HashMap, HashSet,
};
use std::ops::Deref;

type KEY = dyn AsRef<[u8]>;

const STAKE_ACCOUNT_PREFIX: [u8; 1] = [0x1];
const DELEGATION_PREFIX: [u8; 1] = [0x2];

const METADATA_KEY: &[u8; 8] = b"Metadata";
const STAKEHOLDERS_KEY: &[u8; 12] = b"Stakeholders";
const CANDIDATES_KEY: &[u8; 10] = b"Candidates";
const NEXT_VALIDATORS_KEY: &[u8; 14] = b"NextValidators";
const CURRENT_VALIDATORS_KEY: &[u8; 17] = b"CurrentValidators";
const JAIL_KEY: &[u8; 4] = b"Jail";
const BANNED_KEY: &[u8; 6] = b"Banned";

fn prefix_public_key(prefix: &[u8], key: &Public) -> Vec<u8> {
    [prefix, key.as_ref()].concat()
}

fn remove_key(key: &KEY) {
    substorage().remove(key)
}

fn load_with_key<T: DeserializeOwned>(key: &KEY) -> Option<T> {
    substorage().get(key).map(deserialize)
}

fn write_with_key<T: Serialize>(key: &KEY, data: T) {
    substorage().set(key, serialize(data))
}

#[derive(Serialize, Deserialize)]
pub struct Metadata {
    pub seq: u64,
    pub current_term_id: u64,
    pub last_term_finished_block_num: u64,
    pub params: Params,
    pub term_params: Params,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Params {
    pub term_seconds: u64,
    pub nomination_expiration: u64,
    pub custody_period: u64,
    pub release_period: u64,
    pub max_num_of_validators: usize,
    pub min_num_of_validators: usize,
    pub delegation_threshold: StakeQuantity,
    pub min_deposit: DepositQuantity,
    pub max_candidate_metadata_size: usize,

    pub era: u64,
}

impl Metadata {
    pub fn load() -> Self {
        load_with_key(METADATA_KEY).expect("Params must be exist")
    }

    pub fn save(self) {
        write_with_key(METADATA_KEY, self)
    }

    pub fn update_params(&mut self, metadata_seq: u64, new_params: Params) -> Result<(), Error> {
        if self.seq != metadata_seq {
            Err(Error::InvalidMetadataSeq(Mismatch {
                found: metadata_seq,
                expected: self.seq,
            }))
        } else {
            self.params = new_params;
            self.seq += 1;
            Ok(())
        }
    }

    pub fn update_term_params(&mut self) {
        self.term_params = self.params;
    }
}

pub struct StakeAccount<'a> {
    pub public: &'a Public,
    pub balance: StakeQuantity,
}

impl<'a> StakeAccount<'a> {
    pub fn load(public: &'a Public) -> Self {
        StakeAccount {
            public,
            balance: load_with_key(&prefix_public_key(&STAKE_ACCOUNT_PREFIX, public)).unwrap_or_default(),
        }
    }

    pub fn save(self) {
        write_with_key(&prefix_public_key(&STAKE_ACCOUNT_PREFIX, self.public), self.balance)
    }

    pub fn subtract_balance(&mut self, quantity: StakeQuantity) -> Result<(), Error> {
        if self.balance < quantity {
            Err(Error::InsufficientStakes(Insufficient {
                required: quantity,
                actual: self.balance,
            }))
        } else {
            self.balance -= quantity;
            Ok(())
        }
    }

    pub fn add_balance(&mut self, quantity: StakeQuantity) -> Result<(), Error> {
        self.balance += quantity;
        Ok(())
    }
}

pub struct Delegation<'a> {
    pub delegator: &'a Public,
    delegatees: BTreeMap<Public, StakeQuantity>,
}

impl<'a> Delegation<'a> {
    pub fn load(delegator: &'a Public) -> Self {
        Delegation {
            delegator,
            delegatees: load_with_key(&prefix_public_key(&DELEGATION_PREFIX, delegator)).unwrap_or_default(),
        }
    }

    pub fn save(self) {
        let Delegation {
            delegator,
            delegatees,
        } = self;
        write_with_key(delegator, delegatees)
    }

    pub fn add_quantity(&mut self, delegatee: Public, quantity: StakeQuantity) -> Result<(), Error> {
        if quantity != 0 {
            *self.delegatees.entry(delegatee).or_insert(0) += quantity;
        }
        Ok(())
    }

    pub fn sub_quantity(&mut self, delegatee: Public, quantity: StakeQuantity) -> Result<(), Error> {
        if quantity != 0 {
            if let Entry::Occupied(mut entry) = self.delegatees.entry(delegatee) {
                let delegation = entry.get();
                match delegation.cmp(&quantity) {
                    Ordering::Greater => {
                        *entry.get_mut() -= quantity;
                        Ok(())
                    }
                    Ordering::Equal => {
                        entry.remove();
                        Ok(())
                    }
                    Ordering::Less => Err(Error::InsufficientStakes(Insufficient {
                        required: quantity,
                        actual: *delegation,
                    })),
                }
            } else {
                Err(Error::DelegateeNotFoundInCandidates(delegatee))
            }
        } else {
            Ok(())
        }
    }

    pub fn get_quantity(&self, delegatee: &Public) -> StakeQuantity {
        self.delegatees.get(delegatee).cloned().unwrap_or(0)
    }

    pub fn into_iter(self) -> btree_map::IntoIter<Public, StakeQuantity> {
        self.delegatees.into_iter()
    }

    pub fn sum(&self) -> u64 {
        self.delegatees.values().sum()
    }
}

pub struct Stakeholders(BTreeSet<Public>);

impl Stakeholders {
    pub fn load() -> Stakeholders {
        Stakeholders(load_with_key(STAKEHOLDERS_KEY).unwrap_or_default())
    }

    pub fn save(self) {
        let key = STAKEHOLDERS_KEY;
        if !self.0.is_empty() {
            write_with_key(key, self.0)
        } else {
            remove_key(key)
        }
    }

    pub fn delegatees() -> HashMap<Public, StakeQuantity> {
        Stakeholders::load().0.into_iter().fold(HashMap::new(), |mut map, stakeholder| {
            let delegation = Delegation::load(&stakeholder);
            delegation.into_iter().for_each(|(delegatee, quantity)| {
                *map.entry(delegatee).or_default() += quantity;
            });
            map
        })
    }

    pub fn update_by_increased_balance(&mut self, account: &StakeAccount) {
        if account.balance > 0 {
            self.0.insert(*account.public);
        }
    }

    pub fn update_by_decreased_balance(&mut self, account: &StakeAccount, delegation: &Delegation) {
        assert!(account.public == delegation.delegator);
        if account.balance == 0 && delegation.sum() == 0 {
            self.0.remove(account.public);
        }
    }

    pub fn iter(&self) -> btree_set::Iter<'_, Public> {
        self.0.iter()
    }
}

pub struct NextValidators(Vec<Validator>);
impl NextValidators {
    pub fn load() -> Self {
        NextValidators(load_with_key(NEXT_VALIDATORS_KEY).unwrap_or_default())
    }

    pub fn save(self) {
        write_with_key(NEXT_VALIDATORS_KEY, self.0)
    }

    pub fn elect() -> Self {
        let Params {
            delegation_threshold,
            max_num_of_validators,
            min_num_of_validators,
            min_deposit,
            ..
        } = Metadata::load().term_params;
        assert!(max_num_of_validators >= min_num_of_validators);
        let mut validators = Candidates::prepare_validators(min_deposit);
        // validators are now sorted in descending order of (delegation, deposit, priority)
        validators.reverse();

        {
            let banned = Banned::load();
            validators.iter().for_each(|validator| {
                let public = &validator.pubkey();
                assert!(!banned.is_banned(&public), "{:?} is banned public", public);
            });
        }

        validators.truncate(max_num_of_validators);

        if validators.len() < min_num_of_validators {
            println!(
                "There must be something wrong. validators.len() < min_num_of_validators, {} < {}",
                validators.len(),
                min_num_of_validators
            );
        }

        let (minimum, rest) = validators.split_at(min_num_of_validators.min(validators.len()));
        let over_threshold = rest.iter().filter(|c| c.delegation >= delegation_threshold);

        let mut result: Vec<_> = minimum.iter().chain(over_threshold).cloned().collect();
        result.reverse();
        NextValidators(result)
    }

    pub fn update_weight(&mut self, block_author: &Public) {
        let min_delegation = self.min_delegation();
        for Validator {
            weight,
            pubkey,
            ..
        } in self.0.iter_mut().rev()
        {
            if pubkey == block_author {
                // block author
                *weight = weight.saturating_sub(min_delegation);
                break
            }
            // neglecting validators
            *weight = weight.saturating_sub(min_delegation * 2);
        }
        if self.0.iter().all(|validator| validator.weight == 0) {
            self.0.iter_mut().for_each(Validator::reset);
        }
        self.0.sort_unstable();
    }

    pub fn delegation(&self, pubkey: &Public) -> Option<StakeQuantity> {
        self.0.iter().find(|validator| validator.pubkey() == pubkey).map(|validator| validator.delegation)
    }

    fn min_delegation(&self) -> StakeQuantity {
        self.0.iter().map(|validator| validator.delegation).min().expect("There must be at least one validator")
    }
}

impl Deref for NextValidators {
    type Target = Vec<Validator>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<NextValidators> for Vec<Validator> {
    fn from(val: NextValidators) -> Self {
        val.0
    }
}

pub struct CurrentValidators(Vec<Validator>);
impl CurrentValidators {
    pub fn load() -> Self {
        CurrentValidators(load_with_key(CURRENT_VALIDATORS_KEY).unwrap_or_default())
    }

    pub fn save(self) {
        let key = CURRENT_VALIDATORS_KEY;
        if !self.is_empty() {
            write_with_key(key, self.0)
        } else {
            remove_key(key)
        }
    }

    pub fn update(&mut self, validators: Vec<Validator>) {
        self.0 = validators;
    }

    pub fn publics(&self) -> Vec<Public> {
        self.0.iter().rev().map(|v| *v.pubkey()).collect()
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

pub struct Candidates(Vec<Candidate>);

impl Candidates {
    pub fn load() -> Self {
        Candidates(load_with_key(CANDIDATES_KEY).unwrap_or_default())
    }

    pub fn save(self) {
        write_with_key(CANDIDATES_KEY, self.0)
    }

    fn prepare_validators(min_deposit: DepositQuantity) -> Vec<Validator> {
        let Candidates(candidates) = Self::load();
        let delegations = Stakeholders::delegatees();
        let mut result =
            candidates.into_iter().filter(|c| c.deposit >= min_deposit).fold(Vec::new(), |mut vec, candidate| {
                let public = &candidate.pubkey;
                if let Some(&delegation) = delegations.get(public) {
                    vec.push(Validator::new(delegation, candidate.deposit, candidate.pubkey));
                }
                vec
            });
        result.sort_by_key(|v| (v.delegation, v.deposit));
        result
    }

    pub fn get_candidate(&self, account: &Public) -> Option<&Candidate> {
        self.0.iter().find(|&c| &c.pubkey == account)
    }

    pub fn add_deposit(
        &mut self,
        pubkey: &Public,
        quantity: DepositQuantity,
        nomination_ends_at: u64,
        metadata: Bytes,
    ) {
        if let Some(candidate) = self.0.iter_mut().find(|c| c.pubkey == *pubkey) {
            candidate.deposit += quantity;
            candidate.nomination_ends_at = max(candidate.nomination_ends_at, nomination_ends_at);
            candidate.metadata = metadata;
        } else {
            self.0.push(Candidate {
                pubkey: *pubkey,
                deposit: quantity,
                nomination_ends_at,
                metadata,
            })
        };
    }

    pub fn renew_candidates(
        &mut self,
        validators: &NextValidators,
        nomination_ends_at: u64,
        inactive_validators: &[Public],
        banned: &Banned,
    ) {
        let to_renew: HashSet<_> = validators
            .iter()
            .map(|validator| validator.pubkey())
            .filter(|pubkey| !inactive_validators.contains(pubkey))
            .collect();

        for candidate in self.0.iter_mut().filter(|c| to_renew.contains(&c.pubkey)) {
            let public = &candidate.pubkey;
            assert!(!banned.is_banned(public), "{:?} is banned public", public);
            candidate.nomination_ends_at = nomination_ends_at;
        }

        let to_reprioritize: Vec<_> =
            self.0.iter().filter(|c| to_renew.contains(&c.pubkey)).map(|c| c.pubkey).collect();

        self.reprioritize(to_reprioritize);
    }

    pub fn drain_expired_candidates(&mut self, term_index: u64) -> Vec<Candidate> {
        let (expired, retained): (Vec<_>, Vec<_>) = self.0.drain(..).partition(|c| c.nomination_ends_at <= term_index);
        self.0 = retained;
        expired
    }

    pub fn remove(&mut self, public: &Public) -> Option<Candidate> {
        if let Some(index) = self.0.iter().position(|c| &c.pubkey == public) {
            Some(self.0.remove(index))
        } else {
            None
        }
    }

    /// reprioritize candidates in the order of last updated time
    fn reprioritize(&mut self, targets: Vec<Public>) {
        let (mut old, mut renewed): (Vec<_>, Vec<_>) = self.0.drain(..).partition(|c| !targets.contains(&c.pubkey));
        old.append(&mut renewed);
        self.0 = old;
    }
}

pub struct Jail(BTreeMap<Public, Prisoner>);

impl Jail {
    pub fn load() -> Self {
        let prisoners: Vec<Prisoner> = load_with_key(JAIL_KEY).unwrap_or_default();
        Jail(prisoners.into_iter().map(|p| (p.pubkey, p)).collect())
    }

    pub fn save(self) {
        if !self.0.is_empty() {
            let vectorized: Vec<Prisoner> = self.0.into_iter().map(|(_, p)| p).collect();
            write_with_key(JAIL_KEY, vectorized)
        }
    }

    pub fn get_prisoner(&self, public: &Public) -> Option<&Prisoner> {
        self.0.get(public)
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

    pub fn remove(&mut self, public: &Public) -> Option<Prisoner> {
        self.0.remove(public)
    }

    pub fn try_release(&mut self, public: &Public, term_index: u64) -> ReleaseResult {
        match self.0.entry(*public) {
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

    pub fn drain_released_prisoners(&mut self, term_index: u64) -> Vec<Prisoner> {
        let (released, retained): (Vec<_>, Vec<_>) =
            self.0.values().cloned().partition(|c| c.released_at <= term_index);
        self.0 = retained.into_iter().map(|c| (c.pubkey, c)).collect();
        released
    }
}

pub struct Banned(BTreeSet<Public>);

impl Banned {
    pub fn load() -> Self {
        Banned(load_with_key(BANNED_KEY).unwrap_or_default())
    }

    #[allow(dead_code)]
    pub fn save(self) {
        write_with_key(BANNED_KEY, self.0)
    }

    #[allow(dead_code)]
    pub fn add(&mut self, public: Public) {
        self.0.insert(public);
    }

    pub fn is_banned(&self, public: &Public) -> bool {
        self.0.contains(public)
    }
}

pub fn get_stakes() -> HashMap<Public, u64> {
    let stakeholders = Stakeholders::load();
    stakeholders
        .iter()
        .map(|stakeholder| {
            let account = StakeAccount::load(stakeholder);
            let delegation = Delegation::load(stakeholder);
            (*stakeholder, account.balance + delegation.sum())
        })
        .collect()
}
