// Copyright 2019-2020 Kodebox, Inc.
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

use super::ValidatorSet;
use crate::client::ConsensusClient;
use crate::consensus::bit_set::BitSet;
use crate::consensus::EngineError;
use ckey::{public_to_address, Address, Ed25519Public as Public};
use cstate::{CurrentValidators, NextValidators};
use ctypes::transaction::Validator;
use ctypes::util::unexpected::OutOfBounds;
use ctypes::BlockHash;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

#[derive(Default)]
pub struct DynamicValidator {
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
}

impl DynamicValidator {
    fn next_validators(&self, hash: BlockHash) -> Vec<Validator> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client is not initialized");
        let block_id = hash.into();
        let state = client.state_at(block_id).expect("The next validators must be called on the confirmed block");
        let validators = NextValidators::load_from_state(&state).unwrap();
        let mut validators: Vec<_> = validators.into();
        validators.reverse();
        validators
    }

    fn current_validators(&self, hash: BlockHash) -> Vec<Validator> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client is not initialized");
        let block_id = hash.into();
        let state = client.state_at(block_id).expect("The current validators must be called on the confirmed block");
        let validators = CurrentValidators::load_from_state(&state).unwrap();
        let mut validators: Vec<_> = validators.into();
        validators.reverse();
        validators
    }

    fn validators_pubkey(&self, hash: BlockHash) -> Vec<Public> {
        let validators = self.next_validators(hash);
        validators.into_iter().map(|val| *val.pubkey()).collect()
    }

    fn current_validators_pubkey(&self, hash: BlockHash) -> Vec<Public> {
        self.current_validators(hash).into_iter().map(|val| *val.pubkey()).collect()
    }

    pub fn proposer_index(&self, parent: BlockHash, proposed_view: usize) -> usize {
        let validators = self.next_validators(parent);
        let num_validators = validators.len();
        proposed_view % num_validators
    }

    pub fn get_current(&self, hash: &BlockHash, index: usize) -> Public {
        let validators = self.current_validators_pubkey(*hash);
        let n_validators = validators.len();
        *validators.get(index % n_validators).unwrap()
    }

    pub fn check_enough_votes_with_current(&self, hash: &BlockHash, votes: &BitSet) -> Result<(), EngineError> {
        let validators = self.current_validators(*hash);
        let mut voted_delegation = 0u64;
        let n_validators = validators.len();
        for index in votes.true_index_iter() {
            assert!(index < n_validators);
            let validator = validators.get(index).ok_or_else(|| {
                EngineError::ValidatorNotExist {
                    height: 0, // FIXME
                    index,
                }
            })?;
            voted_delegation += validator.delegation();
        }
        let total_delegation: u64 = validators.iter().map(|v| v.delegation()).sum();
        if voted_delegation * 3 > total_delegation * 2 {
            Ok(())
        } else {
            let threshold = total_delegation as usize * 2 / 3;
            Err(EngineError::BadSealFieldSize(OutOfBounds {
                min: Some(threshold),
                max: Some(total_delegation as usize),
                found: voted_delegation as usize,
            }))
        }
    }
}

impl ValidatorSet for DynamicValidator {
    fn contains(&self, parent: &BlockHash, public: &Public) -> bool {
        self.validators_pubkey(*parent).into_iter().any(|pubkey| pubkey == *public)
    }

    fn contains_address(&self, parent: &BlockHash, address: &Address) -> bool {
        self.validators_pubkey(*parent).into_iter().any(|pubkey| public_to_address(&pubkey) == *address)
    }

    fn get(&self, parent: &BlockHash, index: usize) -> Public {
        let validators = self.validators_pubkey(*parent);
        let n_validators = validators.len();
        assert_ne!(0, n_validators);
        *validators.get(index % n_validators).unwrap()
    }

    fn get_index(&self, parent: &BlockHash, public: &Public) -> Option<usize> {
        self.validators_pubkey(*parent)
            .into_iter()
            .enumerate()
            .find(|(_index, pubkey)| pubkey == public)
            .map(|(index, _)| index)
    }

    fn get_index_by_address(&self, parent: &BlockHash, address: &Address) -> Option<usize> {
        let validators = self.validators_pubkey(*parent);
        validators
            .into_iter()
            .enumerate()
            .find(|(_index, pubkey)| public_to_address(pubkey) == *address)
            .map(|(index, _)| index)
    }

    fn next_block_proposer(&self, parent: &BlockHash, view: u64) -> Address {
        let validators = self.validators_pubkey(*parent);
        let n_validators = validators.len();
        let index = view as usize % n_validators;
        public_to_address(validators.get(index).unwrap())
    }

    fn count(&self, parent: &BlockHash) -> usize {
        self.next_validators(*parent).len()
    }

    fn check_enough_votes(&self, parent: &BlockHash, votes: &BitSet) -> Result<(), EngineError> {
        let validators = self.next_validators(*parent);
        let mut voted_delegation = 0u64;
        let n_validators = validators.len();
        for index in votes.true_index_iter() {
            assert!(index < n_validators);
            let validator = validators.get(index).ok_or_else(|| {
                EngineError::ValidatorNotExist {
                    height: 0, // FIXME
                    index,
                }
            })?;
            voted_delegation += validator.delegation();
        }
        let total_delegation: u64 = validators.iter().map(Validator::delegation).sum();
        if voted_delegation * 3 > total_delegation * 2 {
            Ok(())
        } else {
            let threshold = total_delegation as usize * 2 / 3;
            Err(EngineError::BadSealFieldSize(OutOfBounds {
                min: Some(threshold),
                max: Some(total_delegation as usize),
                found: voted_delegation as usize,
            }))
        }
    }

    /// Allows blockchain state access.
    fn register_client(&self, client: Weak<dyn ConsensusClient>) {
        let mut client_lock = self.client.write();
        assert!(client_lock.is_none());
        *client_lock = Some(client);
    }

    fn current_addresses(&self, hash: &BlockHash) -> Vec<Address> {
        self.current_validators_pubkey(*hash).iter().map(public_to_address).collect()
    }

    fn next_addresses(&self, hash: &BlockHash) -> Vec<Address> {
        self.validators_pubkey(*hash).iter().map(public_to_address).collect()
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use ckey::Ed25519Public as Public;

    use super::*;
    use crate::client::{ConsensusClient, TestBlockChainClient};

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
