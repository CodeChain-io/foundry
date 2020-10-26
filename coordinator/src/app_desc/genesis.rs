// Copyright 2018-2020 Kodebox, Inc.
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

use super::bytes::Bytes;
use serde::Deserialize;

/// Scheme genesis.
#[derive(Debug, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    /// Extra data.
    pub extra_data: Option<Bytes>,
}

#[cfg(test)]
mod tests {
    use super::super::bytes::Bytes;
    use super::Genesis;
    use std::str::FromStr;

    #[test]
    fn genesis_deserialization() {
        let s = r#"{
            "extraData": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
        }"#;
        let deserialized: Genesis = serde_yaml::from_str(s).unwrap();
        assert_eq!(deserialized, Genesis {
            extra_data: Some(
                Bytes::from_str("0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa").unwrap()
            ),
        });
    }
}
