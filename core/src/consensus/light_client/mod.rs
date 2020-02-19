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
use ckey::SchnorrSignature;
pub use ctypes::BlockNumber;
use ctypes::{CompactValidatorSet, Header};
pub use primitives::{H256, H512};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(PartialEq, Debug)]
// (index_in_vset, sign); schnorr scheme
pub struct Seal(Vec<(usize, SchnorrSignature)>);

impl Encodable for Seal {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(self.0.len() * 2);
        for (x, y) in self.0.iter() {
            s.append(x);
            s.append(y);
        }
    }
}

impl Decodable for Seal {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count % 2 == 1 {
            return Err(DecoderError::RlpInvalidLength {
                expected: item_count + 1,
                got: item_count,
            })
        }
        let mut vec = Vec::with_capacity(item_count / 2);
        for i in 0..(item_count / 2) {
            vec.push((rlp.val_at(i * 2)?, rlp.val_at(i * 2 + 1)?));
        }
        Ok(Self(vec))
    }
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
