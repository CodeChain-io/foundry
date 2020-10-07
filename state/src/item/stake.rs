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

use crate::{StakeKeyBuilder, StateResult, TopLevelState, TopState, TopStateView};
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
