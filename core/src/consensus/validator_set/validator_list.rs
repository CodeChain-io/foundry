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
use ckey::{Address, BlsPublic};
use ctypes::util::unexpected::OutOfBounds;
use ctypes::BlockHash;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

/// Validator set containing a known set of public keys.
pub struct RoundRobinValidator {
    validators: Vec<(Address, BlsPublic)>,
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
}

impl RoundRobinValidator {
    pub fn new(validators: Vec<(Address, BlsPublic)>) -> Self {
        RoundRobinValidator {
            validators,
            client: Default::default(),
        }
    }
}

impl ValidatorSet for RoundRobinValidator {
    fn contains_public(&self, _bh: &BlockHash, public: &BlsPublic) -> bool {
        self.validators.iter().any(|(_a, p)| p == public)
    }

    fn contains_address(&self, _bh: &BlockHash, address: &Address) -> bool {
        self.validators.iter().any(|(a, _p)| a == address)
    }

    fn get_public(&self, _bh: &BlockHash, index: usize) -> BlsPublic {
        let validator_n = self.validators.len();
        assert_ne!(0, validator_n, "Cannot operate with an empty validator set.");
        (*self.validators.get(index % validator_n).expect("There are validator_n authorities; taking number modulo validator_n gives number in validator_n range; qed")).1
    }

    fn get_address(&self, _bh: &BlockHash, index: usize) -> Address {
        let validator_n = self.validators.len();
        assert_ne!(0, validator_n, "Cannot operate with an empty validator set.");
        (*self.validators.get(index % validator_n).expect("There are validator_n authorities; taking number modulo validator_n gives number in validator_n range; qed")).0
    }

    fn get_index(&self, _bh: &BlockHash, public: &BlsPublic) -> Option<usize> {
        self.validators.iter().position(|(_a, p)| p == public)
    }

    fn get_index_by_address(&self, _bh: &BlockHash, address: &Address) -> Option<usize> {
        self.validators.iter().position(|(a, _p)| a == address)
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
            self.get_address(&parent, proposer_index)
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
        self.validators.iter().map(|(a, _p)| *a).collect()
    }

    fn current_addresses(&self, _hash: &BlockHash) -> Vec<Address> {
        self.validators.iter().map(|(a, _p)| *a).collect()
    }

    fn next_addresses(&self, _hash: &BlockHash) -> Vec<Address> {
        self.validators.iter().map(|(a, _p)| *a).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::super::ValidatorSet;
    use super::{Address, BlsPublic, RoundRobinValidator};

    #[test]
    fn validator_set() {
        let a1 = Address::random();
        let p1 = BlsPublic::random();
        let a2 = Address::random();
        let p2 = BlsPublic::random();
        let set = RoundRobinValidator::new(vec![(a1, p1), (a2, p2)]);
        assert!(set.contains_public(&Default::default(), &p1));
        assert_eq!(set.get_public(&Default::default(), 0), p1);
        assert_eq!(set.get_public(&Default::default(), 1), p2);
        assert_eq!(set.get_public(&Default::default(), 2), p1);
    }
}
