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

use crate::consensus::light_client::{ClientState as ChainClientState, UpdateHeader};
use crate::ibc;
use ibc::commitment_23 as commitment;
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

pub type Kind = u8;

// It exists for every blocks, and will be cumulatively stored in the state.
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ConsensusState {
    // This is not used untill we add a misbehavior predicate
    pub validator_set_hash: H256,
    pub state_root: commitment::CommitmentRoot,
}

// It represents set of data that is required to update the client, as ICS said.
// But be careful since the name 'Header' is confusing.
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct Header {
    pub header_proposal: UpdateHeader,
    // This is not used in verification, but part of header. (will be stored in ConsensusState)
    pub state_root: commitment::CommitmentRoot,
}

// Note: We don't store validator set directly but only hash,
// and then receive the actual set every updates. This is for reducing state storage.
#[derive(PartialEq, Debug)]
pub struct ClientState {
    pub raw: ChainClientState,
}

impl Encodable for ClientState {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2);
        s.append(&self.raw.number);
        s.append(&self.raw.next_validator_set_hash);
    }
}

impl Decodable for ClientState {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 2 {
            return Err(DecoderError::RlpInvalidLength {
                got: item_count,
                expected: 2,
            })
        }
        Ok(Self {
            raw: ChainClientState {
                number: rlp.val_at(0)?,
                next_validator_set_hash: rlp.val_at(1)?,
            },
        })
    }
}

pub const KIND_FOUNDRY: Kind = 0_u8;
