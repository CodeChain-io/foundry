// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::hash::H520;
use serde::Deserialize;

/// Tendermint seal.
#[derive(Debug, PartialEq, Deserialize, Default)]
pub struct TendermintSeal {
    /// Seal round.
    pub prev_view: u64,
    /// Proposal seal signature.
    pub cur_view: u64,
    /// Proposal seal signature.
    pub precommits: Vec<H520>,
}

/// Seal variants.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Seal {
    /// Tendermint seal.
    Tendermint(TendermintSeal),
    /// Generic seal.
    #[serde(with = "hex")]
    Generic(Vec<u8>),
}

impl Default for Seal {
    fn default() -> Self {
        Seal::Tendermint(Default::default())
    }
}

#[cfg(test)]
mod tests {
    use super::super::hash::H520;
    use super::{Seal, TendermintSeal};
    use primitives::H520 as Core520;
    use std::str::FromStr;

    #[test]
    fn seal_deserialization() {
        let s = r#"[{
            "generic": "e011bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa"
        },{
            "tendermint": {
                "prev_view": 3,
                "cur_view": 4,
                "precommits": [
                "4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004"
                ]
            }
        }]"#;

        let deserialized: Vec<Seal> = serde_yaml::from_str(s).unwrap();
        assert_eq!(deserialized, vec![
            Seal::Generic(vec![
                0xe0, 0x11, 0xbb, 0xe8, 0xdb, 0x4e, 0x34, 0x7b, 0x4e, 0x8c, 0x93, 0x7c, 0x1c, 0x83, 0x70, 0xe4, 0xb5,
                0xed, 0x33, 0xad, 0xb3, 0xdb, 0x69, 0xcb, 0xdb, 0x7a, 0x38, 0xe1, 0xe5, 0x0b, 0x1b, 0x82, 0xfa,
            ]),
            Seal::Tendermint(TendermintSeal {
                prev_view: 0x3,
                cur_view: 0x4,
                precommits: vec![H520(Core520::from_str("4000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000004").unwrap())]
            }),
        ]);
    }
}
