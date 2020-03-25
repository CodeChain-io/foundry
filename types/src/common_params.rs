// Copyright 2019-2020 Kodebox, Inc.
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

use cjson::scheme::Params;
use ckey::NetworkId;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub struct CommonParams {
    /// Maximum size of extra data.
    max_extra_data_size: usize,
    /// Network id.
    network_id: NetworkId,
    /// Minimum transaction cost.
    min_pay_transaction_cost: u64,
    min_custom_transaction_cost: u64,
    /// Maximum size of block body.
    max_body_size: usize,
    /// Snapshot creation period in unit of block numbers.
    snapshot_period: u64,

    term_seconds: u64,
    nomination_expiration: u64,
    custody_period: u64,
    release_period: u64,
    max_num_of_validators: usize,
    min_num_of_validators: usize,
    delegation_threshold: u64,
    min_deposit: u64,
    max_candidate_metadata_size: usize,

    era: u64,
}

impl CommonParams {
    pub fn max_extra_data_size(&self) -> usize {
        self.max_extra_data_size
    }
    pub fn network_id(&self) -> NetworkId {
        self.network_id
    }
    pub fn min_pay_transaction_cost(&self) -> u64 {
        self.min_pay_transaction_cost
    }
    pub fn min_custom_transaction_cost(&self) -> u64 {
        self.min_custom_transaction_cost
    }
    pub fn max_body_size(&self) -> usize {
        self.max_body_size
    }
    pub fn snapshot_period(&self) -> u64 {
        self.snapshot_period
    }

    pub fn term_seconds(&self) -> u64 {
        self.term_seconds
    }
    pub fn nomination_expiration(&self) -> u64 {
        self.nomination_expiration
    }
    pub fn custody_period(&self) -> u64 {
        self.custody_period
    }
    pub fn release_period(&self) -> u64 {
        self.release_period
    }
    pub fn max_num_of_validators(&self) -> usize {
        self.max_num_of_validators
    }
    pub fn min_num_of_validators(&self) -> usize {
        self.min_num_of_validators
    }
    pub fn delegation_threshold(&self) -> u64 {
        self.delegation_threshold
    }
    pub fn min_deposit(&self) -> u64 {
        self.min_deposit
    }
    pub fn max_candidate_metadata_size(&self) -> usize {
        self.max_candidate_metadata_size
    }

    pub fn era(&self) -> u64 {
        self.era
    }

    pub fn verify(&self) -> Result<(), String> {
        if self.nomination_expiration == 0 {
            return Err("You should set the nomination expiration".to_string())
        }
        if self.custody_period == 0 {
            return Err("You should set the custody period".to_string())
        }
        if self.release_period == 0 {
            return Err("You should set the release period".to_string())
        }
        if self.max_num_of_validators == 0 {
            return Err("You should set the maximum number of validators".to_string())
        }
        if self.min_num_of_validators == 0 {
            return Err("You should set the minimum number of validators".to_string())
        }
        if self.delegation_threshold == 0 {
            return Err("You should set the delegation threshold".to_string())
        }
        if self.min_deposit == 0 {
            return Err("You should set the minimum deposit".to_string())
        }

        if self.min_num_of_validators > self.max_num_of_validators {
            return Err(format!(
                "The minimum number of validators({}) is larger than the maximum number of validators({})",
                self.min_num_of_validators, self.max_num_of_validators
            ))
        }
        if self.custody_period >= self.release_period {
            return Err(format!(
                "The release period({}) should be longer than the custody period({})",
                self.release_period, self.custody_period
            ))
        }

        Ok(())
    }

    pub fn verify_change(&self, current_params: &Self) -> Result<(), String> {
        self.verify()?;
        let current_network_id = current_params.network_id();
        let transaction_network_id = self.network_id();
        if current_network_id != transaction_network_id {
            return Err(format!(
                "The current network id is {} but the transaction tries to change the network id to {}",
                current_network_id, transaction_network_id
            ))
        }
        if self.era < current_params.era {
            return Err(format!("The era({}) shouldn't be less than the current era({})", self.era, current_params.era))
        }
        Ok(())
    }
}

impl From<Params> for CommonParams {
    fn from(p: Params) -> Self {
        Self {
            max_extra_data_size: p.max_extra_data_size.into(),
            network_id: p.network_id,
            min_pay_transaction_cost: p.min_pay_cost.into(),
            min_custom_transaction_cost: p.min_custom_cost.into(),
            max_body_size: p.max_body_size.into(),
            snapshot_period: p.snapshot_period.into(),
            term_seconds: p.term_seconds.into(),
            nomination_expiration: p.nomination_expiration.into(),
            custody_period: p.custody_period.into(),
            release_period: p.release_period.into(),
            max_num_of_validators: p.max_num_of_validators.into(),
            min_num_of_validators: p.min_num_of_validators.into(),
            delegation_threshold: p.delegation_threshold.into(),
            min_deposit: p.min_deposit.into(),
            max_candidate_metadata_size: p.max_candidate_metadata_size.into(),
            era: p.era.map(From::from).unwrap_or_default(),
        }
    }
}

impl From<CommonParams> for Params {
    fn from(p: CommonParams) -> Params {
        #[allow(deprecated)]
        let mut result: Params = Params {
            max_extra_data_size: p.max_extra_data_size().into(),
            network_id: p.network_id(),
            min_pay_cost: p.min_pay_transaction_cost().into(),
            min_custom_cost: p.min_custom_transaction_cost().into(),
            max_body_size: p.max_body_size().into(),
            snapshot_period: p.snapshot_period().into(),
            term_seconds: p.term_seconds().into(),
            nomination_expiration: p.nomination_expiration().into(),
            custody_period: p.custody_period().into(),
            release_period: p.release_period().into(),
            max_num_of_validators: p.max_num_of_validators().into(),
            min_num_of_validators: p.min_num_of_validators().into(),
            delegation_threshold: p.delegation_threshold().into(),
            min_deposit: p.min_deposit().into(),
            max_candidate_metadata_size: p.max_candidate_metadata_size().into(),
            era: None,
        };
        let era = p.era();
        if era != 0 {
            result.era = Some(era.into());
        }
        result
    }
}

impl Encodable for CommonParams {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(16)
            .append(&self.max_extra_data_size)
            .append(&self.network_id)
            .append(&self.min_pay_transaction_cost)
            .append(&self.min_custom_transaction_cost)
            .append(&self.max_body_size)
            .append(&self.snapshot_period)
            .append(&self.term_seconds)
            .append(&self.nomination_expiration)
            .append(&self.custody_period)
            .append(&self.release_period)
            .append(&self.max_num_of_validators)
            .append(&self.min_num_of_validators)
            .append(&self.delegation_threshold)
            .append(&self.min_deposit)
            .append(&self.max_candidate_metadata_size)
            .append(&self.era);
    }
}

impl Decodable for CommonParams {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let size = rlp.item_count()?;
        if size != 16 {
            return Err(DecoderError::RlpIncorrectListLen {
                expected: 16,
                got: size,
            })
        }

        let max_extra_data_size = rlp.val_at(0)?;
        let network_id = rlp.val_at(1)?;
        let min_pay_transaction_cost = rlp.val_at(2)?;
        let min_custom_transaction_cost = rlp.val_at(3)?;
        let max_body_size = rlp.val_at(4)?;
        let snapshot_period = rlp.val_at(5)?;

        let term_seconds = rlp.val_at(6)?;
        let nomination_expiration = rlp.val_at(7)?;
        let custody_period = rlp.val_at(8)?;
        let release_period = rlp.val_at(9)?;
        let max_num_of_validators = rlp.val_at(10)?;
        let min_num_of_validators = rlp.val_at(11)?;
        let delegation_threshold = rlp.val_at(12)?;
        let min_deposit = rlp.val_at(13)?;
        let max_candidate_metadata_size = rlp.val_at(14)?;
        let era = rlp.val_at(15)?;

        Ok(Self {
            max_extra_data_size,
            network_id,
            min_pay_transaction_cost,
            min_custom_transaction_cost,
            max_body_size,
            snapshot_period,
            term_seconds,
            nomination_expiration,
            custody_period,
            release_period,
            max_num_of_validators,
            min_num_of_validators,
            delegation_threshold,
            min_deposit,
            max_candidate_metadata_size,
            era,
        })
    }
}

impl CommonParams {
    pub fn default_for_test() -> Self {
        Self::from(Params::default())
    }

    pub fn set_dynamic_validator_params_for_test(
        &mut self,
        term_seconds: u64,
        nomination_expiration: u64,
        custody_period: u64,
        release_period: u64,
        max_num_of_validators: usize,
        min_num_of_validators: usize,
        delegation_threshold: u64,
        min_deposit: u64,
        max_candidate_metadata_size: usize,
    ) {
        self.term_seconds = term_seconds;
        self.nomination_expiration = nomination_expiration;
        self.custody_period = custody_period;
        self.release_period = release_period;

        self.min_num_of_validators = min_num_of_validators;
        self.max_num_of_validators = max_num_of_validators;

        self.delegation_threshold = delegation_threshold;
        self.min_deposit = min_deposit;
        self.max_candidate_metadata_size = max_candidate_metadata_size;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn encode_and_decode_default() {
        rlp_encode_and_decode_test!(CommonParams::default_for_test());
    }

    #[test]
    fn rlp_with_extra_fields() {
        let mut params = CommonParams::default_for_test();
        params.term_seconds = 100;
        params.min_deposit = 123;
        rlp_encode_and_decode_test!(params);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn params_from_json_with_stake_params() {
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
        let params = serde_json::from_str::<Params>(s).unwrap();
        let deserialized = CommonParams::from(params.clone());
        assert_eq!(deserialized.max_extra_data_size, 0x20);
        assert_eq!(deserialized.network_id, "tc".into());
        assert_eq!(deserialized.min_pay_transaction_cost, 10);
        assert_eq!(deserialized.min_custom_transaction_cost, 16);
        assert_eq!(deserialized.max_body_size, 4_194_304);
        assert_eq!(deserialized.snapshot_period, 16_384);
        assert_eq!(deserialized.term_seconds, 3600);
        assert_eq!(deserialized.nomination_expiration, 24);
        assert_eq!(deserialized.custody_period, 25);
        assert_eq!(deserialized.release_period, 26);
        assert_eq!(deserialized.max_num_of_validators, 27);
        assert_eq!(deserialized.min_num_of_validators, 28);
        assert_eq!(deserialized.delegation_threshold, 29);
        assert_eq!(deserialized.min_deposit, 30);
        assert_eq!(deserialized.max_candidate_metadata_size, 31);
        assert_eq!(deserialized.era, 32);

        assert_eq!(params, deserialized.into());
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn params_from_json_with_era() {
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
        let params = serde_json::from_str::<Params>(s).unwrap();
        let deserialized = CommonParams::from(params.clone());
        assert_eq!(deserialized.max_extra_data_size, 0x20);
        assert_eq!(deserialized.network_id, "tc".into());
        assert_eq!(deserialized.min_pay_transaction_cost, 10);
        assert_eq!(deserialized.min_custom_transaction_cost, 16);
        assert_eq!(deserialized.max_body_size, 4_194_304);
        assert_eq!(deserialized.snapshot_period, 16_384);
        assert_eq!(deserialized.term_seconds, 3600);
        assert_eq!(deserialized.nomination_expiration, 24);
        assert_eq!(deserialized.custody_period, 25);
        assert_eq!(deserialized.release_period, 26);
        assert_eq!(deserialized.max_num_of_validators, 27);
        assert_eq!(deserialized.min_num_of_validators, 28);
        assert_eq!(deserialized.delegation_threshold, 29);
        assert_eq!(deserialized.min_deposit, 30);
        assert_eq!(deserialized.max_candidate_metadata_size, 31);
        assert_eq!(deserialized.era, 32);

        assert_eq!(params, deserialized.into());
    }
}
