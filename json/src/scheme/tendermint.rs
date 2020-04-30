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
                    "4cnj73b1X2_K3XI_PJSXAQHZG0BD-VO4-TnhS3WEOy8Ym7UmT1Utc0": {
                        "pubkey": "0x5d05595160b7924e5ecf3f2628b440e601f3a531e92fa81571a70e6c695b2d08",
                        "deposit": 300,
                        "nominationEndsAt": 100,
                        "metadata": "alice"
                    }
                },
                "genesisStakes": {
                    "rjmxg19kCmkCxROEoV0QYsrDpOYsjQwusCtN5_oKMEzk-I6kgtAtc0": {
                        "stake": 100,
                        "delegations": {
                            "fjjh0001AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEtc0": 1,
                            "fjjh0002AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAItc0": 2,
                            "fjjh0003AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMtc0": 3,
                            "fjjh0004AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQtc0": 4,
                            "fjjh0005AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAUtc0": 5,
                            "fjjh0006AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYtc0": 6,
                            "fjjh0007AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAActc0": 7,
                            "fjjh0008AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgtc0": 8
                        }
                    }
                }
            }
        }"#;

        let deserialized: Tendermint = serde_json::from_str(s).unwrap();

        assert_eq!(
            deserialized.params.genesis_candidates,
            [(PlatformAddress::from_str("4cnj73b1X2_K3XI_PJSXAQHZG0BD-VO4-TnhS3WEOy8Ym7UmT1Utc0").unwrap(), Deposit {
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
            (PlatformAddress::from_str("fjjh0001AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAEtc0").unwrap(), 1),
            (PlatformAddress::from_str("fjjh0002AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAItc0").unwrap(), 2),
            (PlatformAddress::from_str("fjjh0003AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAMtc0").unwrap(), 3),
            (PlatformAddress::from_str("fjjh0004AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAQtc0").unwrap(), 4),
            (PlatformAddress::from_str("fjjh0005AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAUtc0").unwrap(), 5),
            (PlatformAddress::from_str("fjjh0006AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAYtc0").unwrap(), 6),
            (PlatformAddress::from_str("fjjh0007AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAActc0").unwrap(), 7),
            (PlatformAddress::from_str("fjjh0008AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAgtc0").unwrap(), 8),
        ]
        .iter()
        .cloned()
        .collect();
        let expected_genesis_stakes = [(
            PlatformAddress::from_str("rjmxg19kCmkCxROEoV0QYsrDpOYsjQwusCtN5_oKMEzk-I6kgtAtc0").unwrap(),
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
