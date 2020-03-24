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
                    "tccq9qvruafmf9vegjhkl0ruunkwp0d4lc8fgxknzh5": {
                        "pubkey": "0x5d05595160b7924e5ecf3f2628b440e601f3a531e92fa81571a70e6c695b2d08",
                        "deposit": 300,
                        "nominationEndsAt": 100,
                        "metadata": "alice"
                    }
                },
                "genesisStakes": {
                    "tccq8qlwpt7xcs9lec3c8tyt3kqxlgsus8q4qp3m6ft": {
                        "stake": 100,
                        "delegations": {
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqyca3rwt": 1,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqgfrhflv": 2,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvxf40sk": 3,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqszkma5z": 4,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5duemmc": 5,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcuzl32l": 6,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqungah99": 7,
                            "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpqc2ul2h": 8
                        }
                    }
                }
            }
        }"#;

        let deserialized: Tendermint = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized.params.genesis_candidates,
            [(PlatformAddress::from_str("tccq9qvruafmf9vegjhkl0ruunkwp0d4lc8fgxknzh5").unwrap(), Deposit {
                pubkey: Public::from_str("5d05595160b7924e5ecf3f2628b440e601f3a531e92fa81571a70e6c695b2d08").unwrap(),
                deposit: 300,
                nomination_ends_at: 100,
                metadata: "alice".to_string(),
            })]
            .iter()
            .cloned()
            .collect()
        );

        let expected_delegations = [
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqyca3rwt").unwrap(), 1),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqgfrhflv").unwrap(), 2),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvxf40sk").unwrap(), 3),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqszkma5z").unwrap(), 4),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq5duemmc").unwrap(), 5),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqcuzl32l").unwrap(), 6),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqungah99").unwrap(), 7),
            (PlatformAddress::from_str("tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqpqc2ul2h").unwrap(), 8),
        ]
        .iter()
        .cloned()
        .collect();
        let expected_genesis_stakes =
            [(PlatformAddress::from_str("tccq8qlwpt7xcs9lec3c8tyt3kqxlgsus8q4qp3m6ft").unwrap(), StakeAccount {
                stake: 100,
                delegations: Some(expected_delegations),
            })]
            .iter()
            .cloned()
            .collect();
        assert_eq!(deserialized.params.genesis_stakes, expected_genesis_stakes);
    }
}
