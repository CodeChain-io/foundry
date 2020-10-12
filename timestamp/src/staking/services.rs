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

use super::ServiceHandler;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::module::*;
use ctypes::{ChainParams, CompactValidatorEntry, CompactValidatorSet};

pub type Validators = Vec<Public>;

impl ServiceHandler {
    fn track_validator_set(&self, session: SessionId) -> CompactValidatorSet {
        let validator_set: Validators = self
            .token_manager
            .read()
            .get_owning_accounts_with_issuer(session, &self.config.validator_token_issuer)
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

impl InitGenesis for ServiceHandler {
    fn init_genesis(&self, session: SessionId, config: &[u8]) {
        let initial_validator_set: Vec<String> = serde_cbor::from_slice(config).unwrap();
        let initial_validator_set: Validators =
            initial_validator_set.into_iter().map(|x| std::str::FromStr::from_str(&x).unwrap()).collect();
        let validator_token_issuer = self.config.validator_token_issuer;
        for public in initial_validator_set {
            self.token_manager.read().issue_token(session, &validator_token_issuer, &public).unwrap();
        }
    }
}

impl InitConsensus for ServiceHandler {
    fn init_consensus(&self, session: SessionId) -> (CompactValidatorSet, ChainParams) {
        let validator_set = self.track_validator_set(session);
        let chain_params = ChainParams::default_for_test();
        (validator_set, chain_params)
    }
}

impl UpdateConsensus for ServiceHandler {
    fn update_chain(&self, session: SessionId) -> (Option<CompactValidatorSet>, Option<ChainParams>) {
        let validator_set = self.track_validator_set(session);
        let chain_params = ChainParams::default_for_test();
        (Some(validator_set), Some(chain_params))
    }
}
