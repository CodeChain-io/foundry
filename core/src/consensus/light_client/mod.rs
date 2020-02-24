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

pub mod verification;
pub use self::verification::verify_header;
pub use ctypes::BlockNumber;
use ctypes::{CompactValidatorSet, Header};
pub use primitives::{Bytes, H256, H512};

#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
// (index_in_vset, sign); schnorr scheme
pub struct Seal {
    pub raw: Vec<Bytes>,
}

#[derive(PartialEq, Debug)]
pub struct ClientState {
    pub number: BlockNumber,
    pub next_validator_set_hash: H256,
}

impl ClientState {
    pub fn new(header: &Header) -> Self {
        Self {
            number: header.number(),
            next_validator_set_hash: *header.next_validator_set_hash(),
        }
    }
}

/// All data for updating a light client up to Nth block.
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct UpdateHeader {
    /// 'N'
    pub number: BlockNumber,
    /// Hash of Nth block.
    pub hash: H256,
    /// Seal to Nth block which will be stored in (N+1)th block.
    pub seal: Seal,
    /// Validator set of Nth block which will be stored in (N-1)th state.
    pub validator_set: CompactValidatorSet,
}

impl UpdateHeader {
    pub fn make_client_state(&self) -> ClientState {
        ClientState {
            number: self.number,
            next_validator_set_hash: self.validator_set.hash(),
        }
    }
}
