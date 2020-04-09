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

use crate::db::Key;
use coordinator::types::Event;
use ctypes::{BlockHash, TxHash};
use primitives::H256;
use rlp::{Decodable, Encodable, Rlp, RlpStream};
use std::hash::Hash;
use std::ops::Deref;

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum EventSource {
    Block(BlockHash),
    Transaction(TxHash),
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Events(pub Vec<Event>);

impl Encodable for Events {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_list(&self.0);
    }
}

impl Decodable for Events {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        Ok(Events(rlp.as_list()?))
    }
}

impl Key<Events> for EventSource {
    type Target = H256;

    fn key(&self) -> H256 {
        match self {
            EventSource::Block(hash) => *hash.deref(),
            EventSource::Transaction(hash) => *hash.deref(),
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct EventsWithSource {
    pub source: EventSource,
    pub events: Vec<Event>,
}

#[cfg(test)]
mod tests {
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn encode_and_decode_events() {
        let event1 = Event {
            key: "key1".to_string(),
            value: vec![1, 2, 3, 4, 5],
        };
        let event2 = Event {
            key: "key2".to_string(),
            value: vec![2, 3, 4, 5, 6],
        };
        let event3 = Event {
            key: "key3".to_string(),
            value: vec![3, 4, 5, 6, 7],
        };
        let events = Events(vec![event1, event2, event3]);
        rlp_encode_and_decode_test!(events);
    }
}
