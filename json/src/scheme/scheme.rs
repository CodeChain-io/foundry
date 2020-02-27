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

use super::{Accounts, Engine, Genesis, Params, Shards};
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
    pub shards: Shards,
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
                            ["0xc1f7057e36205fe711c1d645c6c037d10e40e0a8",
          "0x81fc91b26e2bb60e4f1936d63ec3d540507578d38ee3800a691de957419f2f455ce074cb6ef2e179434cf900c6eac9d80af3ac0c7b1b56f118826f33272b8f2cdd62cde37505e2fa3f3f8c89740513c5c055099c02cbed96d26ecef84d224768"],
          ["0xb5f5782552e883ea5b20a3ad0cc4f2f60bd87c39",
          "0xaabd584b58a269c1cf8e790f9561aa0aff86014a121ee3fef76ab36cae0be0e1e942bd242db6cb32321019e74b308a7e01bae6b0c2e41bc8ea751981ec64afa51aa2de5b9bec2e344a109ac58d79492f4bb603586de384d766e8059ac80858ee"]
                        ],
                        "timeoutPropose": 10000,
                        "timeoutPrevote": 10000,
                        "timeoutPrecommit": 10000,
                        "timeoutCommit": 10000
                    }
                }
            },
            "params": {
                "maxExtraDataSize": "0x20",
                "maxAssetSchemeMetadataSize": "0x0400",
                "maxTransferMetadataSize": "0x0100",
                "maxTextContentSize": "0x0200",
                "networkID" : "tc",
                "minPayCost" : 10,
                "minCreateShardCost" : 12,
                "minSetShardOwnersCost" : 13,
                "minSetShardUsersCost" : 14,
                "minWrapCccCost" : 15,
                "minCustomCost" : 16,
                "minMintAssetCost" : 17,
                "minTransferAssetCost" : 18,
                "minChangeAssetSchemeCost" : 19,
                "minIncreaseAssetSupplyCost" : 20,
                "minComposeAssetCost" : 21,
                "minDecomposeAssetCost" : 22,
                "minUnwrapCccCost" : 23,
                "maxBodySize": 4194304,
                "snapshotPeriod": 16384
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
                "score": "0x20000",
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
            },
            "shards": {
            }
        }"#;
        let _deserialized: Scheme = serde_json::from_str(s).unwrap();
        // TODO: validate all fields
    }
}
