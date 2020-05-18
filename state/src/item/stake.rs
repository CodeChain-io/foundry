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
use ctypes::Validators;
use primitives::H256;
use rlp::{decode, encode};
use std::ops::Deref;

lazy_static! {
    pub static ref NEXT_VALIDATORS_KEY: H256 = StakeKeyBuilder::new(1).append(&"Validators").into_key();
    pub static ref CURRENT_VALIDATORS_KEY: H256 = StakeKeyBuilder::new(1).append(&"CurrentValidators").into_key();
}

#[derive(Debug)]
pub struct NextValidators(Validators);
impl NextValidators {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Self> {
        let key = &*NEXT_VALIDATORS_KEY;
        let validators = state
            .action_data(&key)?
            .map(|data| decode(&data).expect("Low level database error. Some issue with disk?"))
            .unwrap_or_default();

        Ok(Self(validators))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = &*NEXT_VALIDATORS_KEY;
        if !self.is_empty() {
            state.update_action_data(&key, encode(&self.0).to_vec())?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
    }
}

impl Deref for NextValidators {
    type Target = Validators;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Validators> for NextValidators {
    fn from(set: Validators) -> Self {
        Self(set)
    }
}

impl From<NextValidators> for Validators {
    fn from(val: NextValidators) -> Self {
        val.0
    }
}

#[derive(Debug)]
pub struct CurrentValidators(Validators);
impl CurrentValidators {
    pub fn load_from_state(state: &TopLevelState) -> StateResult<Self> {
        let key = &*CURRENT_VALIDATORS_KEY;
        let validators = state
            .action_data(&key)?
            .map(|data| decode(&data).expect("Low level database error. Some issue with disk?"))
            .unwrap_or_default();

        Ok(Self(validators))
    }

    pub fn save_to_state(&self, state: &mut TopLevelState) -> StateResult<()> {
        let key = &*CURRENT_VALIDATORS_KEY;
        if !self.is_empty() {
            state.update_action_data(&key, encode(&self.0).to_vec())?;
        } else {
            state.remove_action_data(&key);
        }
        Ok(())
    }

    pub fn update(&mut self, validators: Validators) {
        self.0 = validators;
    }
}

impl Deref for CurrentValidators {
    type Target = Validators;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<CurrentValidators> for Validators {
    fn from(val: CurrentValidators) -> Self {
        val.0
    }
}
