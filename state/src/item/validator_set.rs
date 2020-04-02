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

use super::stake::{CurrentValidators, NextValidators};
use crate::{StateResult, TopLevelState};
use ckey::Ed25519Public as Public;
use ctypes::transaction::Validator;
use ctypes::{CompactValidatorEntry, CompactValidatorSet};
use std::ops::Deref;

// Validator information just enough for the host
pub struct SimpleValidator(Validator);

impl SimpleValidator {
    pub fn weight(&self) -> u64 {
        self.0.delegation()
    }

    pub fn pubkey(&self) -> &Public {
        self.0.pubkey()
    }
}

// TODO: implementation will be changed as we move NextValidators into a separate module
pub struct NextValidatorSet(NextValidators);

impl NextValidatorSet {
    pub fn create_compact_validator_set(&self) -> CompactValidatorSet {
        self.0.create_compact_validator_set()
    }

    pub fn load_from_state(state: &TopLevelState) -> StateResult<Self> {
        Ok(NextValidatorSet(NextValidators::load_from_state(state)?))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        self.0.save_to_state(state)
    }
}

impl Deref for NextValidatorSet {
    type Target = Vec<Validator>;

    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}

impl From<NextValidatorSet> for Vec<SimpleValidator> {
    fn from(validator_set: NextValidatorSet) -> Self {
        validator_set.0.iter().map(|val| SimpleValidator(*val)).collect()
    }
}

// TODO: implementation will be changed as we move CurrentValidators into a separate module
pub struct CurrentValidatorSet(CurrentValidators);

impl CurrentValidatorSet {
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

    pub fn load_from_state(state: &TopLevelState) -> StateResult<Self> {
        Ok(CurrentValidatorSet(CurrentValidators::load_from_state(state)?))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        self.0.save_to_state(state)
    }

    pub fn update(&mut self, next_validator_set: NextValidatorSet) {
        self.0.update(next_validator_set.clone());
    }
}

impl From<CurrentValidatorSet> for Vec<SimpleValidator> {
    fn from(validator_set: CurrentValidatorSet) -> Self {
        validator_set.0.iter().map(|val| SimpleValidator(*val)).collect()
    }
}
