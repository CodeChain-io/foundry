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

use super::{Accounts, Engine, Genesis, Params};
use crate::uint::Uint;
use serde_json::Error;
use std::io::Read;

/// Scheme deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Scheme {
    /// Scheme name.
    pub name: String,
    /// Special fork name.
    pub data_dir: Option<String>,
    /// Engine.
    pub engine: Engine,
    /// Scheme params.
    pub params: Params,
    /// Genesis header.
    pub genesis: Genesis,
    /// Genesis state.
    pub accounts: Accounts,
    pub shards: Uint,
    /// Boot nodes.
    pub nodes: Option<Vec<String>>,
}

impl Scheme {
    /// Loads test from json.
    pub fn load<R>(reader: R) -> Result<Self, Error>
    where
        R: Read, {
        serde_json::from_reader(reader)
    }
}

#[cfg(test)]
mod tests {
    use super::Scheme;

    #[test]
    fn spec_deserialization() {
        let s = r#"{
            "name": "Morden",
            "dataDir": "morden",
            "engine": {
                "tendermint": {
                    "params": {
                        "validators" : [
                            "0x6f57729dbeeae75cb180984f0bf65c56f822135c47337d68a0aef41d7f932375",
                            "0xe3c20d46302d0ce9db2c48619486db2f7f65726e438bcbaaf548ff2671d93c9e"
                        ],
                        "timeoutPropose": 10000,
                        "timeoutPrevote": 10000,
                        "timeoutPrecommit": 10000,
                        "timeoutCommit": 10000,
                        "genesisStakes": {
                          "tccqy9xjqk9zwz2zhgsvt9v8f8x9jxsct4s9dx707s2xpxwf7yw5jpdqurmyde": {
                            "stake": 100,
                            "delegations": {
                              "tccq90kljkawglne9yhqyqajx6qg0u48w8e88s5kavy8vh33xa4ye842kfxyqu": 10
                            }
                          },
                          "tccq90kljkawglne9yhqyqajx6qg0u48w8e88s5kavy8vh33xa4ye842kfxyqu": {
                            "stake": 100
                          }
                        },
                        "genesisCandidates": {
                          "tccq90kljkawglne9yhqyqajx6qg0u48w8e88s5kavy8vh33xa4ye842kfxyqu": {
                            "pubkey": "0x6f57729dbeeae75cb180984f0bf65c56f822135c47337d68a0aef41d7f932375",
                            "deposit": 20,
                            "nominationEndsAt": 100,
                            "metadata": "alice"
                          }
                        }
                    }
                }
            },
            "params": {
                "maxExtraDataSize": "0x20",
                "maxTransferMetadataSize": "0x0100",
                "maxTextContentSize": "0x0200",
                "networkID" : "tc",
                "maxBodySize": 4194304,
                "snapshotPeriod": 16384,
                "termSeconds": 3600,
                "nominationExpiration": 24,
                "custodyPeriod": 25,
                "releasePeriod": 26,
                "maxNumOfValidators": 27,
                "minNumOfValidators": 28,
                "delegationThreshold": 29,
                "minDeposit": 30,
                "maxCandidateMetadataSize": 31
            },
            "genesis": {
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
                "timestamp": "0x00",
                "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
            },
            "nodes": [
            "enode://b1217cbaa440e35ed471157123fe468e19e8b5ad5bedb4b1fdbcbdab6fb2f5ed3e95dd9c24a22a79fdb2352204cea207df27d92bfd21bfd41545e8b16f637499@104.44.138.37:30303"
            ],
            "accounts": {
                "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqzzut2uq": { "balance": "1", "seq": "1048576" },
                "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqy7ng0qh": { "balance": "1", "seq": "1048576" },
                "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqxdkfvn6": { "balance": "1", "seq": "1048576" },
                "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqg0dw93s": { "balance": "1", "seq": "1048576" },
                "tccq8t6d5nxsd7pckgnswusmq6sdzu76kxa808t6m3gtygltrjqeeqncfggwh3": { "balance": "1606938044258990275541962092341162602522202993782792835301376", "seq": "1048576" }
            },
            "shards": 1
        }"#;
        let _deserialized: Scheme = serde_json::from_str(s).unwrap();
        // TODO: validate all fields
    }
}
