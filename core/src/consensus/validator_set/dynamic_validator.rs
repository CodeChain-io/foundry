// Copyright 2019 Kodebox, Inc.
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

use super::{RoundRobinValidator, ValidatorSet};
use crate::client::ConsensusClient;
use crate::consensus::bit_set::BitSet;
use crate::consensus::stake::{CurrentValidators, NextValidators, PreviousValidators, Validator};
use crate::consensus::EngineError;
use ckey::{public_to_address, Address, Ed25519Public as Public};
use ctypes::util::unexpected::OutOfBounds;
use ctypes::BlockHash;
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

/// Validator set containing a known set of public keys.
pub struct DynamicValidator {
    initial_list: RoundRobinValidator,
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
}

impl DynamicValidator {
    pub fn new(initial_validators: Vec<Public>) -> Self {
        DynamicValidator {
            initial_list: RoundRobinValidator::new(initial_validators),
            client: Default::default(),
        }
    }

    fn next_validators(&self, hash: BlockHash) -> Option<Vec<Validator>> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client is not initialized");
        let block_id = hash.into();
        let term_id = client.current_term_id(block_id).expect(
            "valdators() is called when creating a block or verifying a block.
            Minor creates a block only when the parent block is imported.
            The n'th block is verified only when the parent block is imported.",
        );
        if term_id == 0 {
            return None
        }
        let state = client.state_at(block_id)?;
        let validators = NextValidators::load_from_state(&state).unwrap();
        if validators.is_empty() {
            None
        } else {
            let mut validators: Vec<_> = validators.into();
            validators.reverse();
            Some(validators)
        }
    }

    fn current_validators(&self, hash: BlockHash) -> Option<Vec<Validator>> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client is not initialized");
        let block_id = hash.into();
        let term_id = client.current_term_id(block_id).expect(
            "valdators() is called when creating a block or verifying a block.
            Minor creates a block only when the parent block is imported.
            The n'th block is verified only when the parent block is imported.",
        );
        if term_id == 0 {
            return None
        }
        let state = client.state_at(block_id)?;
        let validators = CurrentValidators::load_from_state(&state).unwrap();
        if validators.is_empty() {
            None
        } else {
            let mut validators: Vec<_> = validators.into();
            validators.reverse();
            Some(validators)
        }
    }

    fn previous_validators(&self, hash: BlockHash) -> Option<Vec<Validator>> {
        let client: Arc<dyn ConsensusClient> =
            self.client.read().as_ref().and_then(Weak::upgrade).expect("Client is not initialized");
        let block_id = hash.into();
        let term_id = client.current_term_id(block_id).expect(
            "valdators() is called when creating a block or verifying a block.
            Minor creates a block only when the parent block is imported.
            The n'th block is verified only when the parent block is imported.",
        );
        if term_id == 0 {
            return None
        }
        let state = client.state_at(block_id)?;
        let validators = PreviousValidators::load_from_state(&state).unwrap();
        if validators.is_empty() {
            None
        } else {
            let mut validators: Vec<_> = validators.into();
            validators.reverse();
            Some(validators)
        }
    }

    fn validators_pubkey(&self, hash: BlockHash) -> Option<Vec<Public>> {
        self.next_validators(hash).map(|validators| validators.into_iter().map(|val| *val.pubkey()).collect())
    }

    fn current_validators_pubkey(&self, hash: BlockHash) -> Option<Vec<Public>> {
        self.current_validators(hash).map(|validators| validators.into_iter().map(|val| *val.pubkey()).collect())
    }

    fn previous_validators_pubkey(&self, hash: BlockHash) -> Option<Vec<Public>> {
        self.previous_validators(hash).map(|validators| validators.into_iter().map(|val| *val.pubkey()).collect())
    }

    pub fn proposer_index(&self, parent: BlockHash, prev_proposer_index: usize, proposed_view: usize) -> usize {
        if let Some(validators) = self.next_validators(parent) {
            let num_validators = validators.len();
            proposed_view % num_validators
        } else {
            let num_validators = self.initial_list.count(&parent);
            (prev_proposer_index + proposed_view + 1) % num_validators
        }
    }

    pub fn get_current(&self, hash: &BlockHash, index: usize) -> Option<Public> {
        let validators = self.current_validators_pubkey(*hash)?;
        let n_validators = validators.len();
        Some(*validators.get(index % n_validators).unwrap())
    }

    pub fn check_enough_votes_with_current(&self, hash: &BlockHash, votes: &BitSet) -> Result<(), EngineError> {
        if let Some(validators) = self.current_validators(*hash) {
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
        } else {
            let client = self.client.read().as_ref().and_then(Weak::upgrade).expect("Client is not initialized");
            let header = client.block_header(&(*hash).into()).unwrap();
            self.check_enough_votes(&header.parent_hash(), votes)
        }
    }
}

impl ValidatorSet for DynamicValidator {
    fn contains(&self, parent: &BlockHash, public: &Public) -> bool {
        if let Some(validators) = self.validators_pubkey(*parent) {
            validators.into_iter().any(|pubkey| pubkey == *public)
        } else {
            self.initial_list.contains(parent, public)
        }
    }

    fn contains_address(&self, parent: &BlockHash, address: &Address) -> bool {
        if let Some(validators) = self.validators_pubkey(*parent) {
            validators.into_iter().any(|pubkey| public_to_address(&pubkey) == *address)
        } else {
            self.initial_list.contains_address(parent, address)
        }
    }

    fn get(&self, parent: &BlockHash, index: usize) -> Public {
        if let Some(validators) = self.validators_pubkey(*parent) {
            let n_validators = validators.len();
            *validators.get(index % n_validators).unwrap()
        } else {
            self.initial_list.get(parent, index)
        }
    }

    fn get_index(&self, parent: &BlockHash, public: &Public) -> Option<usize> {
        if let Some(validators) = self.validators_pubkey(*parent) {
            validators.into_iter().enumerate().find(|(_index, pubkey)| pubkey == public).map(|(index, _)| index)
        } else {
            self.initial_list.get_index(parent, public)
        }
    }

    fn get_index_by_address(&self, parent: &BlockHash, address: &Address) -> Option<usize> {
        if let Some(validators) = self.validators_pubkey(*parent) {
            validators
                .into_iter()
                .enumerate()
                .find(|(_index, pubkey)| public_to_address(pubkey) == *address)
                .map(|(index, _)| index)
        } else {
            self.initial_list.get_index_by_address(parent, address)
        }
    }

    fn next_block_proposer(&self, parent: &BlockHash, view: u64) -> Option<Address> {
        if let Some(validators) = self.validators_pubkey(*parent) {
            let n_validators = validators.len();
            let index = view as usize % n_validators;
            Some(public_to_address(validators.get(index).unwrap()))
        } else {
            self.initial_list.next_block_proposer(parent, view)
        }
    }

    fn count(&self, parent: &BlockHash) -> usize {
        if let Some(validators) = self.next_validators(*parent) {
            validators.len()
        } else {
            self.initial_list.count(parent)
        }
    }

    fn check_enough_votes(&self, parent: &BlockHash, votes: &BitSet) -> Result<(), EngineError> {
        if let Some(validators) = self.next_validators(*parent) {
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
        } else {
            self.initial_list.check_enough_votes(parent, votes)
        }
    }

    /// Allows blockchain state access.
    fn register_client(&self, client: Weak<dyn ConsensusClient>) {
        self.initial_list.register_client(Weak::clone(&client));
        let mut client_lock = self.client.write();
        assert!(client_lock.is_none());
        *client_lock = Some(client);
    }

    fn previous_addresses(&self, hash: &BlockHash) -> Vec<Address> {
        if let Some(validators) = self.previous_validators_pubkey(*hash) {
            validators.iter().map(public_to_address).collect()
        } else {
            self.initial_list.previous_addresses(hash)
        }
    }

    fn current_addresses(&self, hash: &BlockHash) -> Vec<Address> {
        if let Some(validators) = self.current_validators_pubkey(*hash) {
            validators.iter().map(public_to_address).collect()
        } else {
            self.initial_list.next_addresses(hash)
        }
    }

    fn next_addresses(&self, hash: &BlockHash) -> Vec<Address> {
        if let Some(validators) = self.validators_pubkey(*hash) {
            validators.iter().map(public_to_address).collect()
        } else {
            self.initial_list.next_addresses(hash)
        }
    }
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;
    use std::sync::Arc;

    use ckey::Ed25519Public as Public;

    use super::super::ValidatorSet;
    use super::DynamicValidator;
    use crate::client::{ConsensusClient, TestBlockChainClient};

    #[test]
    fn validator_set() {
        let a1 = Public::from_str("6f57729dbeeae75cb180984f0bf65c56f822135c47337d68a0aef41d7f932375").unwrap();
        let a2 = Public::from_str("e3c20d46302d0ce9db2c48619486db2f7f65726e438bcbaaf548ff2671d93c9e").unwrap();
        let set = DynamicValidator::new(vec![a1, a2]);
        let test_client: Arc<dyn ConsensusClient> = Arc::new({
            let mut client = TestBlockChainClient::new();
            client.term_id = Some(1);
            client
        });
        set.register_client(Arc::downgrade(&test_client));
        assert!(set.contains(&Default::default(), &a1));
        assert_eq!(set.get(&Default::default(), 0), a1);
        assert_eq!(set.get(&Default::default(), 1), a2);
        assert_eq!(set.get(&Default::default(), 2), a1);
    }
}
