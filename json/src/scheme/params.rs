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
use ckey::NetworkId;

/// Scheme params.
#[derive(Debug, Default, PartialEq, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Params {
    /// Maximum size of extra data.
    pub max_extra_data_size: Uint,
    /// Network id.
    #[serde(rename = "networkID")]
    pub network_id: NetworkId,

    /// Minimum transaction cost.
    pub min_pay_cost: Uint,
    pub min_custom_cost: Uint,

    /// Maximum size of block body.
    pub max_body_size: Uint,
    /// Snapshot creation period in unit of block numbers.
    pub snapshot_period: Uint,

    pub term_seconds: Uint,
    pub nomination_expiration: Uint,
    pub custody_period: Uint,
    pub release_period: Uint,
    pub max_num_of_validators: Uint,
    pub min_num_of_validators: Uint,
    pub delegation_threshold: Uint,
    pub min_deposit: Uint,
    pub max_candidate_metadata_size: Uint,

    /// A monotonically increasing number to denote the consensus version.
    /// It is increased when we fork.
    pub era: Option<Uint>,
}

#[cfg(test)]
mod tests {
    use super::Params;

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn params_deserialization() {
        let s = r#"{
            "maxExtraDataSize": "0x20",
            "networkID" : "tc",
            "minPayCost" : 10,
            "minCustomCost" : 16,
            "maxBodySize" : 4194304,
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
        }"#;

        let deserialized: Params = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized.max_extra_data_size, 0x20.into());
        assert_eq!(deserialized.network_id, "tc".into());
        assert_eq!(deserialized.min_pay_cost, 10.into());
        assert_eq!(deserialized.min_custom_cost, 16.into());
        assert_eq!(deserialized.max_body_size, 4_194_304.into());
        assert_eq!(deserialized.snapshot_period, 16_384.into());
        assert_eq!(deserialized.term_seconds, 3600.into());
        assert_eq!(deserialized.nomination_expiration, 24.into());
        assert_eq!(deserialized.custody_period, 25.into());
        assert_eq!(deserialized.release_period, 26.into());
        assert_eq!(deserialized.max_num_of_validators, 27.into());
        assert_eq!(deserialized.min_num_of_validators, 28.into());
        assert_eq!(deserialized.delegation_threshold, 29.into());
        assert_eq!(deserialized.min_deposit, 30.into());
        assert_eq!(deserialized.max_candidate_metadata_size, 31.into());
        assert_eq!(deserialized.era, None);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn params_deserialization_with_era() {
        let s = r#"{
            "maxExtraDataSize": "0x20",
            "networkID" : "tc",
            "minPayCost" : 10,
            "minCustomCost" : 16,
            "maxBodySize" : 4194304,
            "snapshotPeriod": 16384,
            "termSeconds": 3600,
            "nominationExpiration": 24,
            "custodyPeriod": 25,
            "releasePeriod": 26,
            "maxNumOfValidators": 27,
            "minNumOfValidators": 28,
            "delegationThreshold": 29,
            "minDeposit": 30,
            "maxCandidateMetadataSize": 31,
            "era": 32
        }"#;

        let deserialized: Params = serde_json::from_str(s).unwrap();
        assert_eq!(deserialized.max_extra_data_size, 0x20.into());
        assert_eq!(deserialized.network_id, "tc".into());
        assert_eq!(deserialized.min_pay_cost, 10.into());
        assert_eq!(deserialized.min_custom_cost, 16.into());
        assert_eq!(deserialized.max_body_size, 4_194_304.into());
        assert_eq!(deserialized.snapshot_period, 16_384.into());
        assert_eq!(deserialized.term_seconds, 3600.into());
        assert_eq!(deserialized.nomination_expiration, 24.into());
        assert_eq!(deserialized.custody_period, 25.into());
        assert_eq!(deserialized.release_period, 26.into());
        assert_eq!(deserialized.max_num_of_validators, 27.into());
        assert_eq!(deserialized.min_num_of_validators, 28.into());
        assert_eq!(deserialized.delegation_threshold, 29.into());
        assert_eq!(deserialized.min_deposit, 30.into());
        assert_eq!(deserialized.max_candidate_metadata_size, 31.into());
        assert_eq!(deserialized.era, Some(32.into()));
    }
}
