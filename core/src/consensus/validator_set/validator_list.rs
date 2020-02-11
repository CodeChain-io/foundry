// Copyright 2018-2019 Kodebox, Inc.
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

use super::super::BitSet;
use super::ValidatorSet;
use crate::client::ConsensusClient;
use crate::consensus::EngineError;
use crate::types::BlockId;
use ckey::{public_to_address, Address, Ed25519Public as Public};
use ctypes::util::unexpected::OutOfBounds;
use ctypes::BlockHash;
use parking_lot::RwLock;
use std::collections::HashSet;
use std::sync::{Arc, Weak};

/// Validator set containing a known set of public keys.
pub struct RoundRobinValidator {
    validators: Vec<Public>,
    addresses: HashSet<Address>,
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
}

impl RoundRobinValidator {
    pub fn new(validators: Vec<Public>) -> Self {
        let addresses = validators.iter().map(public_to_address).collect();
        RoundRobinValidator {
            validators,
            addresses,
            client: Default::default(),
        }
    }
}

impl ValidatorSet for RoundRobinValidator {
    fn contains(&self, _bh: &BlockHash, public: &Public) -> bool {
        self.validators.contains(public)
    }

    fn contains_address(&self, _bh: &BlockHash, address: &Address) -> bool {
        self.addresses.contains(address)
    }

    fn get(&self, _bh: &BlockHash, index: usize) -> Public {
        let validator_n = self.validators.len();
        assert_ne!(0, validator_n, "Cannot operate with an empty validator set.");
        *self.validators.get(index % validator_n).expect("There are validator_n authorities; taking number modulo validator_n gives number in validator_n range; qed")
    }

    fn get_index(&self, _bh: &BlockHash, public: &Public) -> Option<usize> {
        self.validators.iter().position(|v| v == public)
    }

    fn get_index_by_address(&self, _bh: &BlockHash, address: &Address) -> Option<usize> {
        self.validators.iter().position(|v| public_to_address(v) == *address)
    }

    fn next_block_proposer(&self, parent: &BlockHash, view: u64) -> Option<Address> {
        let client: Arc<dyn ConsensusClient> = self.client.read().as_ref().and_then(Weak::upgrade)?;
        client.block_header(&BlockId::from(*parent)).map(|header| {
            let proposer = header.author();
            let grand_parent = header.parent_hash();
            let prev_proposer_idx =
                self.get_index_by_address(&grand_parent, &proposer).expect("The proposer must be in the validator set");
            let proposer_index = prev_proposer_idx + 1 + view as usize;
            ctrace!(ENGINE, "Proposer index: {}", proposer_index);
            public_to_address(&self.get(&parent, proposer_index))
        })
    }

    fn count(&self, _bh: &BlockHash) -> usize {
        self.validators.len()
    }

    fn check_enough_votes(&self, parent: &BlockHash, votes: &BitSet) -> Result<(), EngineError> {
        let validator_count = self.count(parent);
        let voted = votes.count();
        if voted * 3 > validator_count * 2 {
            Ok(())
        } else {
            let threshold = validator_count * 2 / 3;
            Err(EngineError::BadSealFieldSize(OutOfBounds {
                min: Some(threshold),
                max: None,
                found: voted,
            }))
        }
    }

    fn register_client(&self, client: Weak<dyn ConsensusClient>) {
        *self.client.write() = Some(client);
    }

    fn previous_addresses(&self, _hash: &BlockHash) -> Vec<Address> {
        self.validators.iter().map(public_to_address).collect()
    }

    fn current_addresses(&self, _hash: &BlockHash) -> Vec<Address> {
        self.validators.iter().map(public_to_address).collect()
    }

    fn next_addresses(&self, _hash: &BlockHash) -> Vec<Address> {
        self.validators.iter().map(public_to_address).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use ckey::Ed25519Public as Public;

    use super::super::ValidatorSet;
    use super::RoundRobinValidator;

    #[test]
    fn validator_set() {
        let a1 = Public::from_str("6f57729dbeeae75cb180984f0bf65c56f822135c47337d68a0aef41d7f932375").unwrap();
        let a2 = Public::from_str("e3c20d46302d0ce9db2c48619486db2f7f65726e438bcbaaf548ff2671d93c9e").unwrap();
        let set = RoundRobinValidator::new(vec![a1, a2]);
        assert!(set.contains(&Default::default(), &a1));
        assert_eq!(set.get(&Default::default(), 0), a1);
        assert_eq!(set.get(&Default::default(), 1), a2);
        assert_eq!(set.get(&Default::default(), 2), a1);
    }
}
