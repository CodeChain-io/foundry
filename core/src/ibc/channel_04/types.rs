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

use crate::ibc::Identifier;
use primitives::Bytes;
use rlp;
use rlp::{DecoderError, Rlp, RlpStream};

#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct Sequence {
    pub raw: u64,
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ChannelState {
    INIT,
    TRYOPEN,
    OPEN,
    CLOSED,
}

impl rlp::Encodable for ChannelState {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(*self as u8));
    }
}

impl rlp::Decodable for ChannelState {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let byte: u8 = rlp.as_val()?;
        match byte {
            0 => Ok(ChannelState::INIT),
            1 => Ok(ChannelState::TRYOPEN),
            2 => Ok(ChannelState::OPEN),
            3 => Ok(ChannelState::CLOSED),
            _ => Err(DecoderError::Custom("Unexpected ChannelState Value")),
        }
    }
}

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ChannelOrder {
    ORDERED,
    UNORDERED,
}

impl rlp::Encodable for ChannelOrder {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(*self as u8));
    }
}

impl rlp::Decodable for ChannelOrder {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let byte: u8 = rlp.as_val()?;
        match byte {
            0 => Ok(ChannelOrder::ORDERED),
            1 => Ok(ChannelOrder::UNORDERED),
            _ => Err(DecoderError::Custom("Unexpected ChannelOrder Value")),
        }
    }
}

#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ChannelEnd {
    pub state: ChannelState,
    pub ordering: ChannelOrder,
    pub counterparty_port_identifier: Identifier,
    pub counterparty_channel_identifier: Identifier,
    pub connection_hops: Vec<Identifier>,
    pub version: Identifier,
}

#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct Packet {
    pub sequence: Sequence,
    pub timeout_height: u64,
    pub source_port: Identifier,
    pub source_channel: Identifier,
    pub dest_port: Identifier,
    pub dest_channel: Identifier,
    pub data: Bytes,
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlp::{self, rlp_encode_and_decode_test};

    #[test]
    fn channel_state() {
        rlp_encode_and_decode_test!(ChannelState::INIT);
    }

    #[test]
    fn channel_order() {
        rlp_encode_and_decode_test!(ChannelOrder::ORDERED);
    }

    #[test]
    fn channel_end() {
        let test = ChannelEnd {
            state: ChannelState::INIT,
            ordering: ChannelOrder::ORDERED,
            counterparty_port_identifier: "Bach".to_owned(),
            counterparty_channel_identifier: "Mozart".to_owned(),
            connection_hops: vec!["Beethoven".to_owned()],
            version: "Schubert".to_owned(),
        };
        rlp_encode_and_decode_test!(test);
    }

    #[test]
    fn packet() {
        let test = Packet {
            sequence: Sequence {
                raw: 0,
            },
            timeout_height: 0,
            source_port: "Schumann".to_owned(),
            source_channel: "Brahms".to_owned(),
            dest_port: "Bruckner".to_owned(),
            dest_channel: "Mahler".to_owned(),
            data: "Schoenberg".to_owned().into_bytes(),
        };
        rlp_encode_and_decode_test!(test);
    }
}
