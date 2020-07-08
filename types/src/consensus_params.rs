// Copyright 2020 Kodebox, Inc.
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
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Serialize, Deserialize, Copy, Clone, Debug, Default, PartialEq)]
pub struct ConsensusParams {
    /// Maximum size of extra data.
    max_extra_data_size: u64,
    /// Network id.
    network_id: NetworkId,
    /// Maximum size of block body.
    max_body_size: u64,
    /// Snapshot creation period in unit of block numbers.
    snapshot_period: u64,

    term_seconds: u64,
}

impl ConsensusParams {
    pub fn max_extra_data_size(&self) -> u64 {
        self.max_extra_data_size
    }
    pub fn network_id(&self) -> NetworkId {
        self.network_id
    }
    pub fn max_body_size(&self) -> u64 {
        self.max_body_size
    }
    pub fn snapshot_period(&self) -> u64 {
        self.snapshot_period
    }
    pub fn term_seconds(&self) -> u64 {
        self.term_seconds
    }

    pub fn default_for_test() -> Self {
        Self {
            max_extra_data_size: 1000,
            network_id: NetworkId::from_str("dt").unwrap(),
            max_body_size: 1000,
            snapshot_period: 1000,
            term_seconds: 1000,
        }
    }
}

impl Encodable for ConsensusParams {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(5)
            .append(&self.max_extra_data_size)
            .append(&self.network_id)
            .append(&self.max_body_size)
            .append(&self.snapshot_period)
            .append(&self.term_seconds);
    }
}

impl Decodable for ConsensusParams {
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
        let term_seconds = rlp.val_at(4)?;

        Ok(Self {
            max_extra_data_size,
            network_id,
            max_body_size,
            snapshot_period,
            term_seconds,
        })
    }
}

impl From<Params> for ConsensusParams {
    fn from(p: Params) -> Self {
        Self {
            max_extra_data_size: p.max_extra_data_size.into(),
            network_id: p.network_id,
            max_body_size: p.max_body_size.into(),
            snapshot_period: p.snapshot_period.into(),
            term_seconds: p.term_seconds.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn encode_and_decode_default() {
        rlp_encode_and_decode_test!(ConsensusParams::default_for_test());
    }

    #[test]
    fn rlp_with_extra_fields() {
        let mut params = ConsensusParams::default_for_test();
        params.max_extra_data_size = 100;
        params.max_body_size = 123;
        rlp_encode_and_decode_test!(params);
    }
}
