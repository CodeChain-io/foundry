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

use crate::stake::StakeKeyBuilder;
use crate::{StateResult, TopLevelState, TopState, TopStateView};
use ctypes::CompactValidatorEntry;
use primitives::H256;

lazy_static! {
    pub static ref NEXT_VALIDATORS_KEY: H256 = StakeKeyBuilder::new(1).append(&"Validators").into_key();
    pub static ref CURRENT_VALIDATORS_KEY: H256 = StakeKeyBuilder::new(1).append(&"CurrentValidators").into_key();
}

#[derive(Debug)]
pub struct StateValidatorSet(Vec<CompactValidatorEntry>);

impl StateValidatorSet {
    pub fn new(validator_set: Vec<CompactValidatorEntry>) -> Self {
        Self(validator_set)
    }

    pub fn to_vec(self) -> Vec<CompactValidatorEntry> {
        self.0
    }

    pub fn hash(&self) -> H256 {
        ccrypto::blake256(&serde_cbor::to_vec(&self.0).unwrap())
    }

    pub fn save_to_state(&self, state: &mut TopLevelState, current: bool) -> StateResult<()> {
        let key = if current {
            &*CURRENT_VALIDATORS_KEY
        } else {
            &*NEXT_VALIDATORS_KEY
        };
        let bytes = serde_cbor::to_vec(&self.0).unwrap();
        state.update_action_data(&key, bytes)?;
        Ok(())
    }

    pub fn load_from_state(state: &TopLevelState, current: bool) -> StateResult<Self> {
        let key = if current {
            &*CURRENT_VALIDATORS_KEY
        } else {
            &*NEXT_VALIDATORS_KEY
        };
        let validator_set = serde_cbor::from_slice(&state.action_data(&key)?.unwrap()).unwrap();
        Ok(Self(validator_set))
    }
}
