// Copyright 2019-2020 Kodebox, Inc.
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

use super::ValidatorSet;
use crate::client::ConsensusClient;
use crate::consensus::bit_set::BitSet;
use crate::consensus::EngineError;
use ckey::Ed25519Public as Public;
use cstate::{CurrentValidatorSet, NextValidatorSet, SimpleValidator};
use ctypes::util::unexpected::OutOfBounds;
use ctypes::BlockHash;
use parking_lot::RwLock;
use std::cmp::Reverse;
use std::sync::{Arc, Weak};

#[derive(Default)]
pub struct DynamicValidator {
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
}

pub struct WeightOrderedValidators(Vec<Public>);

pub struct WeightIndex(usize);

impl WeightOrderedValidators {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: WeightIndex) -> Option<&Public> {
        self.0.get(index.0)
    }
}

impl DynamicValidator {
    fn next_validators(&self, hash: BlockHash) -> Vec<SimpleValidator> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client is not initialized");
        let block_id = hash.into();
        let state = client.state_at(block_id).expect("The next validators must be called on the confirmed block");
        let validators = NextValidatorSet::load_from_state(&state).unwrap();
        validators.into()
    }

    fn current_validators(&self, hash: BlockHash) -> Vec<SimpleValidator> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client is not initialized");
        let block_id = hash.into();
        let state = client.state_at(block_id).expect("The current validators must be called on the confirmed block");
        let validators = CurrentValidatorSet::load_from_state(&state).unwrap();
        validators.into()
    }

    fn validators(&self, hash: BlockHash) -> Vec<Public> {
        let validators = self.next_validators(hash);
        validators.into_iter().map(|val| *val.pubkey()).collect()
    }

    fn validators_order_by_weight(&self, hash: BlockHash) -> WeightOrderedValidators {
        let mut validators = self.next_validators(hash);
        // Should we cache the sorted validator?
        validators.sort_unstable_by_key(|v| {
            (
                Reverse(v.weight()),
                Reverse(v.deposit()),
                v.nominated_at_block_number(),
                v.nominated_at_transaction_index(),
            )
        });
        WeightOrderedValidators(validators.into_iter().map(|val| *val.pubkey()).collect())
    }

    pub fn proposer_index(&self, parent: BlockHash, proposed_view: u64) -> usize {
        let propser = self.next_block_proposer(&parent, proposed_view);
        self.get_index(&parent, &propser).expect("We know propser is included in a validator set")
    }

    pub fn get_current(&self, hash: &BlockHash, index: usize) -> Public {
        let validators = self.current_validators(*hash);
        let n_validators = validators.len();
        *validators.get(index % n_validators).unwrap().pubkey()
    }

    pub fn check_enough_votes_with_current(&self, hash: &BlockHash, votes: &BitSet) -> Result<(), EngineError> {
        let validators = self.current_validators(*hash);
        let mut voted_weight = 0u64;
        let n_validators = validators.len();
        for index in votes.true_index_iter() {
            assert!(index < n_validators);
            let validator = validators.get(index).ok_or_else(|| {
                EngineError::ValidatorNotExist {
                    height: 0, // FIXME
                    index,
                }
            })?;
            voted_weight += validator.weight();
        }
        let total_weight: u64 = validators.iter().map(|v| v.weight()).sum();
        if voted_weight * 3 > total_weight * 2 {
            Ok(())
        } else {
            let threshold = total_weight as usize * 2 / 3;
            Err(EngineError::BadSealFieldSize(OutOfBounds {
                min: Some(threshold),
                max: Some(total_weight as usize),
                found: voted_weight as usize,
            }))
        }
    }
}

impl ValidatorSet for DynamicValidator {
    fn contains(&self, parent: &BlockHash, public: &Public) -> bool {
        self.validators(*parent).into_iter().any(|pubkey| pubkey == *public)
    }

    fn get(&self, parent: &BlockHash, index: usize) -> Public {
        let validators = self.validators(*parent);
        let n_validators = validators.len();
        assert_ne!(0, n_validators);
        *validators.get(index % n_validators).unwrap()
    }

    fn get_index(&self, parent: &BlockHash, public: &Public) -> Option<usize> {
        self.validators(*parent).binary_search(public).ok()
    }

    fn next_block_proposer(&self, parent: &BlockHash, view: u64) -> Public {
        let validators = self.validators_order_by_weight(*parent);
        let n_validators = validators.len();
        let index = WeightIndex(view as usize % n_validators);
        *validators.get(index).unwrap()
    }

    fn count(&self, parent: &BlockHash) -> usize {
        self.next_validators(*parent).len()
    }

    fn check_enough_votes(&self, parent: &BlockHash, votes: &BitSet) -> Result<(), EngineError> {
        let validators = self.next_validators(*parent);
        let mut voted_weight = 0u64;
        let n_validators = validators.len();
        for index in votes.true_index_iter() {
            assert!(index < n_validators);
            let validator = validators.get(index).ok_or_else(|| {
                EngineError::ValidatorNotExist {
                    height: 0, // FIXME
                    index,
                }
            })?;
            voted_weight += validator.weight();
        }
        let total_weight: u64 = validators.iter().map(SimpleValidator::weight).sum();
        if voted_weight * 3 > total_weight * 2 {
            Ok(())
        } else {
            let threshold = total_weight as usize * 2 / 3;
            Err(EngineError::BadSealFieldSize(OutOfBounds {
                min: Some(threshold),
                max: Some(total_weight as usize),
                found: voted_weight as usize,
            }))
        }
    }

    /// Allows blockchain state access.
    fn register_client(&self, client: Weak<dyn ConsensusClient>) {
        let mut client_lock = self.client.write();
        assert!(client_lock.is_none());
        *client_lock = Some(client);
    }

    fn current_validators(&self, hash: &BlockHash) -> Vec<Public> {
        DynamicValidator::current_validators(self, *hash).into_iter().map(|v| *v.pubkey()).collect()
    }

    fn next_validators(&self, hash: &BlockHash) -> Vec<Public> {
        self.validators(*hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::client::{ConsensusClient, TestBlockChainClient};
    use std::str::FromStr;
    use std::sync::Arc;

    #[test]
    fn validator_set() {
        let a1 = Public::from_str("6f57729dbeeae75cb180984f0bf65c56f822135c47337d68a0aef41d7f932375").unwrap();
        let a2 = Public::from_str("e3c20d46302d0ce9db2c48619486db2f7f65726e438bcbaaf548ff2671d93c9e").unwrap();
        let set = DynamicValidator::default();
        let test_client: Arc<dyn ConsensusClient> = Arc::new({
            let mut client = TestBlockChainClient::new();
            client.term_id = Some(1);
            client
        });
        set.register_client(Arc::downgrade(&test_client));
        assert!(!set.contains(&Default::default(), &a1));
        assert!(!set.contains(&Default::default(), &a2));
    }
}
