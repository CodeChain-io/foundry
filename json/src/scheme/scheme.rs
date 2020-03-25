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
                          "tccq8qlwpt7xcs9lec3c8tyt3kqxlgsus8q4qp3m6ft": {
                            "stake": 100,
                            "delegations": {
                              "tccq9qvruafmf9vegjhkl0ruunkwp0d4lc8fgxknzh5": 10
                            }
                          },
                          "tccq9qvruafmf9vegjhkl0ruunkwp0d4lc8fgxknzh5": {
                            "stake": 100
                          }
                        },
                        "genesisCandidates": {
                          "tccq9qvruafmf9vegjhkl0ruunkwp0d4lc8fgxknzh5": {
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
                "minPayCost" : 10,
                "minCustomCost" : 16,
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
                "author": "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhhn9p3",
                "timestamp": "0x00",
                "parentHash": "0x0000000000000000000000000000000000000000000000000000000000000000"
            },
            "nodes": [
            "enode://b1217cbaa440e35ed471157123fe468e19e8b5ad5bedb4b1fdbcbdab6fb2f5ed3e95dd9c24a22a79fdb2352204cea207df27d92bfd21bfd41545e8b16f637499@104.44.138.37:30303"
            ],
            "accounts": {
                "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqyca3rwt": { "balance": "1", "seq": "1048576" },
                "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqgfrhflv": { "balance": "1", "seq": "1048576" },
                "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqvxf40sk": { "balance": "1", "seq": "1048576" },
                "tccqyqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqszkma5z": { "balance": "1", "seq": "1048576" },
                "tccq8txq9uafdg8y2de9m2tdkhsfsj3m9nluq94hyan": { "balance": "1606938044258990275541962092341162602522202993782792835301376", "seq": "1048576" }
            }
        }"#;
        let _deserialized: Scheme = serde_json::from_str(s).unwrap();
        // TODO: validate all fields
    }
}
