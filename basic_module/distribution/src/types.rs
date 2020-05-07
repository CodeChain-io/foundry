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

use ckey::{Ed25519Public as Public, NetworkId};
use ctypes::BlockHash;
use std::collections::BTreeMap;
use std::convert::TryFrom;
pub type BlockNumber = u64;

/// Uniquely identifies block.
#[derive(Debug, PartialEq, Copy, Clone, Hash, Eq)]
pub enum BlockId {
    /// Block's blake256.
    /// Querying by hash is always faster.
    Hash(BlockHash),
    /// Block number within canon blockchain.
    Number(BlockNumber),
    /// Earliest block (genesis).
    Earliest,
    /// Latest mined block.
    Latest,
    /// Parent of latest mined block.
    ParentOfLatest,
}

impl From<BlockHash> for BlockId {
    fn from(hash: BlockHash) -> Self {
        BlockId::Hash(hash)
    }
}

impl From<BlockNumber> for BlockId {
    fn from(number: BlockNumber) -> Self {
        BlockId::Number(number)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Transaction {
    /// Seq.
    pub seq: u64,
    /// Quantity of CCC to be paid as a cost for distributing this transaction to the network.
    pub fee: u64,
    /// Network Id
    pub network_id: NetworkId,

    pub order: u64,

    pub action: Action,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
    DistributeRewards {
        calculated_fees: BTreeMap<Public, u64>,
        block_author_rewards: u64,
    },
    UpdateRewards {
        block_number: u64,
        block_author: Public,
        rewards: BTreeMap<Public, u64>,
    },
}

const MAX_VALIDATOR_SIZE: usize = 800;
const BITSET_SIZE: usize = MAX_VALIDATOR_SIZE / 8;

#[derive(Copy, Clone)]
pub struct BitSet([u8; BITSET_SIZE]);

impl BitSet {
    pub fn is_set(&self, index: usize) -> bool {
        let array_index = index / 8;
        let bit_index = index % 8;

        self.0[array_index] & (1 << bit_index) != 0
    }

    pub fn count(&self) -> usize {
        self.0
            .iter()
            .map(|v| usize::try_from(v.count_ones()).expect("CodeChain doesn't support 16-bits architecture"))
            .sum()
    }

    pub fn true_index_iter(&self) -> BitSetIndexIterator<'_> {
        BitSetIndexIterator {
            index: 0,
            bitset: self,
        }
    }
}

pub struct BitSetIndexIterator<'a> {
    index: usize,
    bitset: &'a BitSet,
}

impl<'a> Iterator for BitSetIndexIterator<'a> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= MAX_VALIDATOR_SIZE {
            return None
        }

        while !self.bitset.is_set(self.index) {
            self.index += 1;

            if self.index >= MAX_VALIDATOR_SIZE {
                return None
            }
        }

        let result = Some(self.index);
        self.index += 1;
        result
    }
}
