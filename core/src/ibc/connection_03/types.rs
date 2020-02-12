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

use rlp;
use rlp::{DecoderError, Rlp, RlpStream};

#[repr(u8)]
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ConnectionState {
    INIT = 0,
    TRYOPEN = 1,
    OPEN = 2,
}

impl rlp::Encodable for ConnectionState {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(*self as u8));
    }
}

impl rlp::Decodable for ConnectionState {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let byte: u8 = rlp.as_val()?;
        match byte {
            0 => Ok(ConnectionState::INIT),
            1 => Ok(ConnectionState::TRYOPEN),
            2 => Ok(ConnectionState::OPEN),
            _ => Err(DecoderError::Custom("Unexpected ConsensusState Value")),
        }
    }
}

// FIXME: current commitment_23::Prefix is too generic.
pub type CommitmentPrefix = String;
pub type Identifier = String;

#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ConnectionEnd {
    pub state: ConnectionState,
    pub counterparty_connection_identifier: Identifier,
    pub counterparty_prefix: CommitmentPrefix,
    pub client_identifier: Identifier,
    pub counterparty_client_identifier: Identifier,
    // FIXME: implement version
}

#[derive(RlpEncodableWrapper, RlpDecodableWrapper, PartialEq, Debug)]
pub struct ConnectionIdentifiersInClient(Vec<Identifier>);

impl ConnectionIdentifiersInClient {
    pub fn add(&mut self, identifier: Identifier) {
        self.0.push(identifier);
    }
}

#[cfg(test)]
mod tests {
    use rlp::{self, rlp_encode_and_decode_test};

    use super::*;

    #[test]
    fn connection_state() {
        rlp_encode_and_decode_test!(ConnectionState::INIT);
    }

    #[test]
    fn connection_end() {
        let connection_end = ConnectionEnd {
            state: ConnectionState::INIT,
            counterparty_connection_identifier: "counterparty_connection_identifier".to_owned(),
            counterparty_prefix: "counterparty_prefix".to_owned(),
            client_identifier: "client_identifier".to_owned(),
            counterparty_client_identifier: "counterparty_client_identifier".to_owned(),
        };
        rlp_encode_and_decode_test!(connection_end);
    }

    #[test]
    fn connection_identifiers_in_client() {
        let identifiers = ConnectionIdentifiersInClient(vec!["a".to_owned(), "b".to_owned()]);
        rlp_encode_and_decode_test!(identifiers);
    }
}
