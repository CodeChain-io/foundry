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

use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

const DATAGRAM_CREATE_CLIENT: u8 = 1;
const DATAGRAM_UPDATE_CLIENT: u8 = 2;

#[derive(Debug, PartialEq)]
pub enum Datagram {
    CreateClient {
        id: String,
        kind: u8,
        consensus_state: Vec<u8>,
    },
    UpdateClient {
        id: String,
        header: Vec<u8>,
    },
}

impl Encodable for Datagram {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            Datagram::CreateClient {
                id,
                kind,
                consensus_state,
            } => {
                s.begin_list(4).append(&DATAGRAM_CREATE_CLIENT).append(id).append(kind).append(consensus_state);
            }
            Datagram::UpdateClient {
                id,
                header,
            } => {
                s.begin_list(3).append(&DATAGRAM_UPDATE_CLIENT).append(id).append(header);
            }
        };
    }
}

impl Decodable for Datagram {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let tag = rlp.val_at(0)?;
        match tag {
            DATAGRAM_CREATE_CLIENT => {
                let item_count = rlp.item_count()?;
                if item_count != 4 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 4,
                        got: item_count,
                    })
                }
                Ok(Datagram::CreateClient {
                    id: rlp.val_at(1)?,
                    kind: rlp.val_at(2)?,
                    consensus_state: rlp.val_at(3)?,
                })
            }
            DATAGRAM_UPDATE_CLIENT => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 3,
                        got: item_count,
                    })
                }
                Ok(Datagram::UpdateClient {
                    id: rlp.val_at(1)?,
                    header: rlp.val_at(2)?,
                })
            }
            _ => Err(DecoderError::Custom("Unexpected IBC Datagram Type")),
        }
    }
}
