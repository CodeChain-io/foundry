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

use crate::{StakeKeyBuilder, StateResult, TopLevelState, TopState, TopStateView};
use ckey::Ed25519Public as Public;
use ctypes::transaction::Validator;
use ctypes::{CompactValidatorEntry, CompactValidatorSet};
use primitives::H256;
use rlp::{decode_list, encode_list};
use std::ops::Deref;
use std::vec;

lazy_static! {
    pub static ref NEXT_VALIDATORS_KEY: H256 = StakeKeyBuilder::new(1).append(&"Validators").into_key();
    pub static ref CURRENT_VALIDATORS_KEY: H256 = StakeKeyBuilder::new(1).append(&"CurrentValidators").into_key();
}

pub type StakeQuantity = u64;

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
        for validator in validators.0.iter_mut().rev() {
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
        validators.0.sort_unstable();
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
        let validators = state.action_data(&key)?.map(|data| decode_list(&data)).unwrap_or_default();

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

    pub fn update(&mut self, validators: Vec<Validator>) {
        self.0 = validators;
    }

    pub fn pubkeys(&self) -> Vec<Public> {
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
