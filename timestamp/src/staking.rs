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

use crate::token::TokenManager;
use ckey::Ed25519Public as Public;
use coordinator::module::*;
use ctypes::{CompactValidatorEntry, CompactValidatorSet, ConsensusParams};
use parking_lot::RwLock;
use primitives::H256;
use std::sync::Arc;

pub type Validators = Vec<Public>;

pub struct Context {
    pub token_manager: RwLock<Arc<dyn TokenManager>>,
    pub validator_token_issuer: H256,
}

impl Context {
    fn track_validator_set(&self) -> CompactValidatorSet {
        let validator_set: Validators = self
            .token_manager
            .read()
            .get_owning_accounts_with_issuer(&self.validator_token_issuer)
            .unwrap()
            .into_iter()
            .collect();
        CompactValidatorSet::new(
            validator_set
                .into_iter()
                .map(|x| CompactValidatorEntry {
                    public_key: x,
                    delegation: 1,
                })
                .collect(),
        )
    }
}

pub trait StakeManager: Send + Sync {
    fn get_validators(&self) -> Validators;
}

impl StakeManager for Context {
    fn get_validators(&self) -> Validators {
        self.token_manager
            .read()
            .get_owning_accounts_with_issuer(&self.validator_token_issuer)
            .unwrap()
            .into_iter()
            .collect()
    }
}

impl InitGenesis for Context {
    fn begin_genesis(&self) {}

    fn init_genesis(&mut self, config: &[u8]) {
        let initial_validator_set: Validators = serde_cbor::from_slice(config).unwrap();
        for public in initial_validator_set {
            self.token_manager.read().issue_token(&self.validator_token_issuer, &public).unwrap();
        }
    }

    fn end_genesis(&self) {}
}

impl InitChain for Context {
    fn init_chain(&self) -> (CompactValidatorSet, ConsensusParams) {
        let validator_set = self.track_validator_set();
        let consensus_params = ConsensusParams::default_for_test();
        (validator_set, consensus_params)
    }
}

impl UpdateChain for Context {
    fn update_chain(&self) -> (CompactValidatorSet, ConsensusParams) {
        let validator_set = self.track_validator_set();
        let consensus_params = ConsensusParams::default_for_test();
        (validator_set, consensus_params)
    }
}
