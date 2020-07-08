// Copyright 2020 Kodebox, Inc.
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

use ckey::Ed25519Public as Public;
use ctypes::BlockHash;
use primitives::Bytes;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Header {
    /// Parent hash.
    parent_hash: BlockHash,
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: u64,
    /// Block author.
    author: Public,
    /// Validators who submitted tendermint Commit for last block.
    last_committed_validators: Vec<Public>,
    /// Block extra data.
    extra_data: Bytes,
}

impl Header {
    pub fn new(
        parent_hash: BlockHash,
        timestamp: u64,
        number: u64,
        author: Public,
        last_committed_validators: Vec<Public>,
        extra_data: Bytes,
    ) -> Self {
        Self {
            parent_hash,
            timestamp,
            number,
            author,
            last_committed_validators,
            extra_data,
        }
    }

    pub fn parent_hash(&self) -> &BlockHash {
        &self.parent_hash
    }

    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }

    pub fn number(&self) -> u64 {
        self.number
    }

    pub fn author(&self) -> &Public {
        &self.author
    }
}
