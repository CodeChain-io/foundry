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

use super::Seal;
use crate::bytes::Bytes;
use crate::hash::H256;
use crate::uint::Uint;
use ckey::PlatformAddress;

/// Scheme genesis.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Genesis {
    /// Seal.
    pub seal: Seal,
    /// Block author, defaults to 0.
    pub author: Option<PlatformAddress>,
    /// Block timestamp, defaults to 0.
    pub timestamp: Option<Uint>,
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
    use super::super::{Seal, TendermintSeal};
    use super::Genesis;
    use crate::bytes::Bytes;
    use crate::hash::{H256, H520};
    use ckey::PlatformAddress;
    use primitives::{H256 as Core256, H520 as Core520};
    use std::str::FromStr;

    #[test]
    fn genesis_deserialization() {
        let s = r#"{
            "seal": {
                "tendermint": {
                    "prev_view": "0x0",
                    "cur_view": "0x0",
                    "precommits": [
                    "0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000"
                    ]
                }
            },
            "author": "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq3e2f0d",
            "timestamp": "0x07",
            "parentHash": "0x9000000000000000000000000000000000000000000000000000000000000000",
            "extraData": "0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa",
            "stateRoot": "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544",
            "nextValidatorSetHash": "0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"
        }"#;
        let deserialized: Genesis = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized, Genesis {
            seal: Seal::Tendermint(TendermintSeal {
                prev_view: 0x0.into(),
                cur_view: 0x0.into(),
                precommits: vec![
                    H520(Core520::from("0x0000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000")),
                ]
            }),
            author: Some(PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq3e2f0d").unwrap()),
            timestamp: Some(0x07.into()),
            parent_hash: Some(H256(Core256::from("0x9000000000000000000000000000000000000000000000000000000000000000"))),
            transactions_root: None,
            state_root: Some(H256(Core256::from("0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"))),
            next_validator_set_hash: Some(H256(Core256::from("0xd7f8974fb5ac78d9ac099b9ad5018bedc2ce0a72dad1827a1709da30580f0544"))),
            extra_data: Some(Bytes::from_str("0x11bbe8db4e347b4e8c937c1c8370e4b5ed33adb3db69cbdb7a38e1e50b1b82fa").unwrap()),
        });
    }
}
