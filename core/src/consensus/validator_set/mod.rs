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

use super::BitSet;
use crate::client::ConsensusClient;
use crate::consensus::EngineError;
use ckey::Ed25519Public as Public;
use ctypes::BlockHash;
use std::sync::Weak;

mod dynamic_validator;

pub use self::dynamic_validator::DynamicValidator;

/// A validator set.
pub trait ValidatorSet: Send + Sync {
    /// Checks if a given public key is a validator,
    /// using underlying, default call mechanism.
    fn contains(&self, parent: &BlockHash, public: &Public) -> bool;

    /// Draws a validator from index modulo number of validators.
    fn get(&self, parent: &BlockHash, index: usize) -> Public;

    /// Draws a validator from nonce modulo number of validators.
    fn get_index(&self, parent: &BlockHash, public: &Public) -> Option<usize>;

    fn next_block_proposer(&self, parent: &BlockHash, view: u64) -> Public;

    /// Returns the current number of validators.
    fn count(&self, parent: &BlockHash) -> usize;

    fn check_enough_votes(&self, parent: &BlockHash, votes: &BitSet) -> Result<(), EngineError>;

    /// Allows blockchain state access.
    fn register_client(&self, _client: Weak<dyn ConsensusClient>) {}

    fn current_validators(&self, _hash: &BlockHash) -> Vec<Public>;

    fn next_validators(&self, _hash: &BlockHash) -> Vec<Public>;
}
