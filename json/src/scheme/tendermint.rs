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

use crate::uint::Uint;
use ckey::{Ed25519Public as Public, PlatformAddress};
use std::collections::HashMap;

/// Tendermint params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TendermintParams {
    /// Propose step timeout in milliseconds.
    pub timeout_propose: Option<Uint>,
    /// Propose step timeout delta in milliseconds.
    pub timeout_propose_delta: Option<Uint>,
    /// Prevote step timeout in milliseconds.
    pub timeout_prevote: Option<Uint>,
    /// Prevote step timeout delta in milliseconds.
    pub timeout_prevote_delta: Option<Uint>,
    /// Precommit step timeout in milliseconds.
    pub timeout_precommit: Option<Uint>,
    /// Precommit step timeout delta in milliseconds.
    pub timeout_precommit_delta: Option<Uint>,
    /// Commit step timeout in milliseconds.
    pub timeout_commit: Option<Uint>,
    /// How much tokens are distributed at Genesis?
    pub genesis_stakes: HashMap<PlatformAddress, StakeAccount>,
    /// allowed past time gap in milliseconds.
    pub allowed_past_timegap: Option<Uint>,
    /// allowed future time gap in milliseconds.
    pub allowed_future_timegap: Option<Uint>,
    /// Genesis candidates.
    pub genesis_candidates: HashMap<PlatformAddress, Deposit>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deposit {
    pub pubkey: Public,
    pub deposit: u64,
    pub nomination_ends_at: u64,
    pub metadata: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakeAccount {
    pub stake: u64,
    pub delegations: Option<HashMap<PlatformAddress, u64>>,
}

/// Tendermint engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Tendermint {
    pub params: TendermintParams,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn deserialization() {
        let s = r#"{
            "params": {
                "genesisCandidates": {
                    "tccq90kljkawglne9yhqyqajx6qg0u48w8e88s5kavy8vh33xa4ye842kfxyqu": {
                        "pubkey": "0x5d05595160b7924e5ecf3f2628b440e601f3a531e92fa81571a70e6c695b2d08",
                        "deposit": 300,
                        "nominationEndsAt": 100,
                        "metadata": "alice"
                    }
                },
                "genesisStakes": {
                    "tccqy9xjqk9zwz2zhgsvt9v8f8x9jxsct4s9dx707s2xpxwf7yw5jpdqurmyde": {
                        "stake": 100,
                        "delegations": {
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzzut2uq": 1,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqy7ng0qh": 2,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxdkfvn6": 3,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqg0dw93s": 4,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq2ug0xza": 5,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvq8vr72": 6,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqwnzdqd8": 7,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsycz367": 8
                        }
                    }
                }
            }
        }"#;

        let deserialized: Tendermint = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized.params.genesis_candidates,
            [(
                PlatformAddress::from_str("tccq90kljkawglne9yhqyqajx6qg0u48w8e88s5kavy8vh33xa4ye842kfxyqu").unwrap(),
                Deposit {
                    pubkey: Public::from_str("5d05595160b7924e5ecf3f2628b440e601f3a531e92fa81571a70e6c695b2d08")
                        .unwrap(),
                    deposit: 300,
                    nomination_ends_at: 100,
                    metadata: "alice".to_string(),
                }
            )]
            .iter()
            .cloned()
            .collect()
        );

        let expected_delegations = [
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzzut2uq").unwrap(), 1),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqy7ng0qh").unwrap(), 2),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxdkfvn6").unwrap(), 3),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqg0dw93s").unwrap(), 4),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq2ug0xza").unwrap(), 5),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvq8vr72").unwrap(), 6),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqwnzdqd8").unwrap(), 7),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqsycz367").unwrap(), 8),
        ]
        .iter()
        .cloned()
        .collect();
        let expected_genesis_stakes = [(
            PlatformAddress::from_str("tccqy9xjqk9zwz2zhgsvt9v8f8x9jxsct4s9dx707s2xpxwf7yw5jpdqurmyde").unwrap(),
            StakeAccount {
                stake: 100,
                delegations: Some(expected_delegations),
            },
        )]
        .iter()
        .cloned()
        .collect();
        assert_eq!(deserialized.params.genesis_stakes, expected_genesis_stakes);
    }
}
