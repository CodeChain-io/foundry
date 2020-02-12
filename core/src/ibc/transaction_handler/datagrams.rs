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

#[repr(u8)]
#[derive(Clone, Copy)]
enum DatagramTag {
    CreateClient = 1,
    UpdateClient = 2,
}

impl Encodable for DatagramTag {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(*self as u8));
    }
}

impl Decodable for DatagramTag {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let byte: u8 = rlp.as_val()?;
        match byte {
            1 => Ok(DatagramTag::CreateClient),
            2 => Ok(DatagramTag::UpdateClient),
            _ => Err(DecoderError::Custom("Unexpected DatagramTag Value")),
        }
    }
}

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
                s.begin_list(4).append(&DatagramTag::CreateClient).append(id).append(kind).append(consensus_state);
            }
            Datagram::UpdateClient {
                id,
                header,
            } => {
                s.begin_list(3).append(&DatagramTag::UpdateClient).append(id).append(header);
            }
        };
    }
}

impl Decodable for Datagram {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let tag = rlp.val_at(0)?;
        match tag {
            DatagramTag::CreateClient => {
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
            DatagramTag::UpdateClient => {
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
        }
    }
}
