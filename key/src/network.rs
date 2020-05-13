// Copyright 2018, 2020 Kodebox, Inc.
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
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::ops::Index;
use std::str::{self, FromStr};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash)]
pub struct NetworkId([u8; 2]);

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        let s = str::from_utf8(&self.0).expect("network_id a valid utf8 string");
        write!(f, "{}", s)
    }
}

impl FromStr for NetworkId {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() != 2 {
            return Err("Invalid network_id length".to_string())
        }
        let mut network_id = [0u8; 2];
        network_id.copy_from_slice(s.as_bytes());
        Ok(NetworkId(network_id))
    }
}

impl From<&'static str> for NetworkId {
    fn from(s: &'static str) -> Self {
        s.parse().unwrap_or_else(|_| panic!("invalid string literal for {}: '{}'", stringify!(Self), s))
    }
}

impl Default for NetworkId {
    fn default() -> Self {
        "tc".into()
    }
}

impl Encodable for NetworkId {
    fn rlp_append(&self, s: &mut RlpStream) {
        let data: String = self.to_string();
        data.rlp_append(s);
    }
}

impl Decodable for NetworkId {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let network_id = String::decode(rlp)?;
        if network_id.len() != 2 {
            return Err(DecoderError::RlpInvalidLength {
                expected: 2,
                got: network_id.len(),
            })
        }
        Ok(network_id.parse().expect("The length of network id is already checked"))
    }
}

impl Serialize for NetworkId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
        let data: String = self.to_string();
        data.serialize(serializer)
    }
}

impl<'a> Deserialize<'a> for NetworkId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'a>, {
        let data = String::deserialize(deserializer)?;
        data.parse().map_err(|_| Error::custom("Invalid network_id"))
    }
}

impl Index<usize> for NetworkId {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialization() {
        let s = r#""tc""#;
        let network_id: NetworkId = serde_json::from_str(s).unwrap();
        assert_eq!(NetworkId::from("tc"), network_id);
    }
}
