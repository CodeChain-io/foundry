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
use super::hash::H256;
use super::seal::Seal;
use ckey::PlatformAddress;
use serde::Deserialize;

/// Scheme genesis.
#[derive(Debug, PartialEq, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    /// Seal.
    pub seal: Seal,
    /// Block author, defaults to 0.
    pub author: Option<PlatformAddress>,
    /// Block timestamp, defaults to 0.
    pub timestamp: Option<u64>,
    /// Parent hash, defaults to 0.
    pub parent_hash: Option<H256>,
    /// Transactions root.
    pub transactions_root: Option<H256>,
    /// State root.
    pub state_root: Option<H256>,
    /// Next validator set hash.
    pub next_validator_set_hash: Option<H256>,
    /// Extra data.
    pub extra_data: Option<Bytes>,
}

#[cfg(test)]
mod tests {
    use super::super::bytes::Bytes;
    use super::super::hash::{H256, H520};
    use super::super::seal::TendermintSeal;
    use super::Genesis;
    use super::Seal;
    use ckey::PlatformAddress;
    use primitives::{H256 as Core256, H520 as Core520};
    use std::str::FromStr;

    #[test]
    fn genesis_deserialization() {
        let s = r#"{
            "seal": {
                "tendermint": {
                    "prev_view": 0,
                    "cur_view": 0,
                    "precommits": [
                    "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                    ]
                }
            },
            "author": "fjjh0000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAtc0",
            "timestamp": 0x07,
            "parentHash": "0x9000000000000000000000000000000000000000000000000000000000000000",
            "extraData": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
            "stateRoot": "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544",
            "nextValidatorSetHash": "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"
        }"#;
        let deserialized: Genesis = serde_yaml::from_str(s).unwrap();
        assert_eq!(deserialized, Genesis {
            seal: Seal::Tendermint(TendermintSeal {
                prev_view: 0x0,
                cur_view: 0x0,
                precommits: vec![
                    H520(Core520::from_str("0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000").unwrap()),
                ]
            }),
            author: Some(PlatformAddress::from_str("fjjh0000AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAtc0").unwrap()),
            timestamp: Some(0x07),
            parent_hash: Some(H256(Core256::from_str("9000000000000000000000000000000000000000000000000000000000000000").unwrap())),
            transactions_root: None,
            state_root: Some(H256(Core256::from_str("d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544").unwrap())),
            next_validator_set_hash: Some(H256(Core256::from_str("d7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544").unwrap())),
            extra_data: Some(Bytes::from_str("0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa").unwrap()),
        });
    }
}
