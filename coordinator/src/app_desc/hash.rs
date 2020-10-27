// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// Copyright 2020 Kodebox, Inc.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

//! Lenient hash json deserialization for test json files.

use primitives::H256;
use rustc_hex::FromHexError;
use serde::de::Visitor;
use serde::de::{Error, Unexpected};
use serde::Deserializer;
use std::fmt;
use std::fmt::Formatter;
use std::str::FromStr;

pub fn deserialize_h256<'de, D: Deserializer<'de>>(deserializer: D) -> Result<H256, D::Error> {
    struct H256Visitor;

    impl<'de> Visitor<'de> for H256Visitor {
        type Value = H256;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            write!(formatter, "64 hexadecimals representing a H256")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            let hash = H256::from_str(v).map_err(|e| match e {
                FromHexError::InvalidHexCharacter(_char, _usize) => {
                    let message = &*format!("{:?}", e);
                    E::invalid_value(Unexpected::Str(v), &message)
                }
                FromHexError::InvalidHexLength => E::invalid_length(v.len(), &"64 hex decimals"),
            })?;
            Ok(hash)
        }
    }
    deserializer.deserialize_str(H256Visitor)
}

#[cfg(test)]
mod test {
    use super::deserialize_h256;
    use primitives::H256;
    use serde::Deserialize;
    use std::str::FromStr;

    #[derive(PartialEq, Deserialize, Debug)]
    struct H256Wrapper(#[serde(deserialize_with = "deserialize_h256")] H256);

    #[test]
    fn hash_deserialization() {
        let s = r#"["0000000000000000000000000000000000000000000000000000000000000000", "5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae"]"#;
        let deserialized: Vec<H256Wrapper> = serde_yaml::from_str(s).unwrap();
        assert_eq!(deserialized, vec![
            H256Wrapper(H256::zero()),
            H256Wrapper(H256::from_str("5a39ed1020c04d4d84539975b893a4e7c53eab6c2965db8bc3468093a31bc5ae").unwrap()),
        ]);
    }
}
