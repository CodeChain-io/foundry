// Copyright 2019-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use cjson::scheme::Params;
use ckey::NetworkId;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub struct CommonParams {
    /// Maximum size of extra data.
    max_extra_data_size: usize,
    /// Network id.
    network_id: NetworkId,
    /// Maximum size of block body.
    max_body_size: usize,
    /// Snapshot creation period in unit of block numbers.
    snapshot_period: u64,

    era: u64,
}

impl CommonParams {
    pub fn max_extra_data_size(&self) -> usize {
        self.max_extra_data_size
    }
    pub fn network_id(&self) -> NetworkId {
        self.network_id
    }
    pub fn max_body_size(&self) -> usize {
        self.max_body_size
    }
    pub fn snapshot_period(&self) -> u64 {
        self.snapshot_period
    }

    pub fn era(&self) -> u64 {
        self.era
    }

    pub fn verify(&self) -> Result<(), String> {
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
            max_body_size: p.max_body_size.into(),
            snapshot_period: p.snapshot_period.into(),
            era: p.era.map(From::from).unwrap_or_default(),
        }
    }
}

impl From<CommonParams> for Params {
    fn from(p: CommonParams) -> Params {
        let mut result: Params = Params {
            max_extra_data_size: p.max_extra_data_size().into(),
            network_id: p.network_id(),
            max_body_size: p.max_body_size().into(),
            snapshot_period: p.snapshot_period().into(),
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
        s.begin_list(5)
            .append(&self.max_extra_data_size)
            .append(&self.network_id)
            .append(&self.max_body_size)
            .append(&self.snapshot_period)
            .append(&self.era);
    }
}

impl Decodable for CommonParams {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let size = rlp.item_count()?;
        if size != 5 {
            return Err(DecoderError::RlpIncorrectListLen {
                expected: 5,
                got: size,
            })
        }

        let max_extra_data_size = rlp.val_at(0)?;
        let network_id = rlp.val_at(1)?;
        let max_body_size = rlp.val_at(2)?;
        let snapshot_period = rlp.val_at(3)?;

        let era = rlp.val_at(4)?;

        Ok(Self {
            max_extra_data_size,
            network_id,
            max_body_size,
            snapshot_period,
            era,
        })
    }
}

impl CommonParams {
    pub fn default_for_test() -> Self {
        Self::from(Params::default())
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
    fn params_from_json_with_era() {
        let s = r#"{
            "maxExtraDataSize": "0x20",
            "networkID" : "tc",
            "maxBodySize" : 4194304,
            "snapshotPeriod": 16384,
            "era": 32
        }"#;
        let params = serde_json::from_str::<Params>(s).unwrap();
        let deserialized = CommonParams::from(params.clone());
        assert_eq!(deserialized.max_extra_data_size, 0x20);
        assert_eq!(deserialized.network_id, "tc".into());
        assert_eq!(deserialized.max_body_size, 4_194_304);
        assert_eq!(deserialized.snapshot_period, 16_384);
        assert_eq!(deserialized.era, 32);

        assert_eq!(params, deserialized.into());
    }
}
