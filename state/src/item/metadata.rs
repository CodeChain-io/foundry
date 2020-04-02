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

use crate::CacheableItem;
use ctypes::{CommonParams, ConsensusParams, StorageId};
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metadata {
    number_of_modules: StorageId,
    last_term_finished_block_num: u64,
    current_term_id: u64,
    seq: u64,
    params: CommonParams,
    consensus_params: ConsensusParams,
}

impl Metadata {
    pub fn new(params: CommonParams, consensus_params: ConsensusParams) -> Self {
        Self {
            number_of_modules: 0,
            last_term_finished_block_num: 0,
            current_term_id: 0,
            seq: 0,
            params,
            consensus_params,
        }
    }

    pub fn number_of_modules(&self) -> &StorageId {
        &self.number_of_modules
    }

    pub fn add_module(&mut self) -> StorageId {
        let r = self.number_of_modules;
        self.number_of_modules += 1;
        r
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn increase_seq(&mut self) {
        self.seq += 1;
    }

    pub fn params(&self) -> &CommonParams {
        &self.params
    }

    pub fn set_params(&mut self, params: CommonParams) {
        self.params = params;
    }

    pub fn consensus_params(&self) -> &ConsensusParams {
        &self.consensus_params
    }

    pub fn set_consensus_params(&mut self, consensus_params: ConsensusParams) {
        self.consensus_params = consensus_params;
    }

    pub fn increase_term_id(&mut self, last_term_finished_block_num: u64) {
        assert!(self.last_term_finished_block_num < last_term_finished_block_num);
        self.last_term_finished_block_num = last_term_finished_block_num;
        self.current_term_id += 1;
    }

    pub fn last_term_finished_block_num(&self) -> u64 {
        self.last_term_finished_block_num
    }

    pub fn current_term_id(&self) -> u64 {
        self.current_term_id
    }
}

impl CacheableItem for Metadata {
    type Address = MetadataAddress;

    fn is_null(&self) -> bool {
        self.number_of_modules == 0
    }
}

const PREFIX: u8 = super::Prefix::Metadata as u8;

impl Encodable for Metadata {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(7)
            .append(&PREFIX)
            .append(&self.number_of_modules)
            .append(&self.last_term_finished_block_num)
            .append(&self.current_term_id)
            .append(&self.seq)
            .append(&self.params)
            .append(&self.consensus_params);
    }
}

impl Decodable for Metadata {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 7 {
            return Err(DecoderError::RlpInvalidLength {
                got: item_count,
                expected: 7,
            })
        }

        let prefix = rlp.val_at::<u8>(0)?;
        if PREFIX != prefix {
            cdebug!(STATE, "{} is not an expected prefix for metadata", prefix);
            return Err(DecoderError::Custom("Unexpected prefix"))
        }
        let number_of_modules = rlp.val_at(1)?;

        let last_term_finished_block_num = rlp.val_at(2)?;
        let current_term_id = rlp.val_at(3)?;
        let seq = rlp.val_at(4)?;
        let params = rlp.val_at(5)?;

        let consensus_params = rlp.val_at(6)?;

        Ok(Self {
            number_of_modules,
            last_term_finished_block_num,
            current_term_id,
            seq,
            params,
            consensus_params,
        })
    }
}

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MetadataAddress(H256);

impl_address!(TOP, MetadataAddress, PREFIX);

impl MetadataAddress {
    pub fn new() -> Self {
        Self::from_transaction_hash(H256::from_slice(b"metadata address"), 0)
    }
}

#[cfg(test)]
mod tests {
    use ctypes::CommonParams;
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn parse_fail_return_none() {
        let hash = {
            let mut hash;
            loop {
                hash = H256::random();
                if hash[0] == PREFIX {
                    continue
                }
                break
            }
            hash
        };
        let address = MetadataAddress::from_hash(hash);
        assert!(address.is_none());
    }

    #[test]
    fn parse_return_some() {
        let hash = {
            let mut hash = H256::random();
            hash[0] = PREFIX;
            hash
        };
        let address = MetadataAddress::from_hash(hash);
        assert_eq!(Some(MetadataAddress(hash)), address);
    }

    #[test]
    fn metadata_with_0_seq() {
        let metadata = Metadata::default();
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn metadata_without_term_with_seq() {
        let metadata = Metadata {
            number_of_modules: 7,
            last_term_finished_block_num: 0,
            current_term_id: 0,
            seq: 3,
            params: CommonParams::default_for_test(),
            consensus_params: ConsensusParams::default_for_test(),
        };
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn metadata_with_term_without_seq() {
        let metadata = Metadata {
            number_of_modules: 7,
            last_term_finished_block_num: 0,
            current_term_id: 0,
            seq: 0,
            params: CommonParams::default_for_test(),
            consensus_params: ConsensusParams::default_for_test(),
        };
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn metadata_with_term_and_seq() {
        let metadata = Metadata {
            number_of_modules: 7,
            last_term_finished_block_num: 1,
            current_term_id: 100,
            seq: 3,
            params: CommonParams::default_for_test(),
            consensus_params: ConsensusParams::default_for_test(),
        };
        rlp_encode_and_decode_test!(metadata);
    }
}
