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

use std::ops::Deref;

use ctypes::{BlockHash, BlockNumber, TxHash};
use primitives::{H256, H264, U256};

use crate::db::Key;
use crate::types::TransactionId;

/// Represents index of extra data in database
#[derive(Copy, Debug, Hash, Eq, PartialEq, Clone)]
enum ExtrasIndex {
    /// Block details index
    BlockDetails = 0,
    /// Block hash index
    BlockHash = 1,
    /// Transaction address index
    TransactionAddress = 2,
    // (Reserved) = 3,
    // (Reserved) = 4,
    // (Reserved) = 5,
}

fn with_index(hash: &H256, i: ExtrasIndex) -> H264 {
    let mut result = H264::default();
    result[0] = i as u8;
    (*result)[1..].copy_from_slice(hash);
    result
}

pub struct BlockNumberKey([u8; 5]);

impl Deref for BlockNumberKey {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Key<BlockHash> for BlockNumber {
    type Target = BlockNumberKey;

    fn key(&self) -> Self::Target {
        let mut result = [0u8; 5];
        result[0] = ExtrasIndex::BlockHash as u8;
        result[1] = (self >> 24) as u8;
        result[2] = (self >> 16) as u8;
        result[3] = (self >> 8) as u8;
        result[4] = *self as u8;
        BlockNumberKey(result)
    }
}

impl Key<BlockDetails> for BlockHash {
    type Target = H264;

    fn key(&self) -> H264 {
        with_index(self, ExtrasIndex::BlockDetails)
    }
}

impl Key<TransactionAddress> for TxHash {
    type Target = H264;

    fn key(&self) -> H264 {
        with_index(self, ExtrasIndex::TransactionAddress)
    }
}

/// Familial details concerning a block
#[derive(Debug, Clone, RlpEncodable, RlpDecodable)]
pub struct BlockDetails {
    /// Block number
    pub number: BlockNumber,
    /// Total score of the block and all its parents
    pub total_score: U256,
    /// Parent block hash
    pub parent: BlockHash,
}

/// Represents address of certain transaction within block
#[derive(Debug, PartialEq, Clone, Copy, RlpEncodable, RlpDecodable)]
pub struct TransactionAddress {
    /// Block hash
    pub block_hash: BlockHash,
    /// Transaction index within the block
    pub index: usize,
}

impl From<TransactionAddress> for TransactionId {
    fn from(addr: TransactionAddress) -> Self {
        TransactionId::Location(addr.block_hash.into(), addr.index)
    }
}
