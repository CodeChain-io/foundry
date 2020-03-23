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
use ctypes::{CommonParams, ShardId, StorageId, TxHash};
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metadata {
    number_of_shards: ShardId,
    number_of_modules: StorageId,
    number_of_initial_shards: ShardId,
    hashes: Vec<TxHash>,
    last_term_finished_block_num: u64,
    current_term_id: u64,
    seq: u64,
    params: CommonParams,
    term_params: CommonParams,
}

impl Metadata {
    pub fn new(number_of_shards: ShardId, params: CommonParams) -> Self {
        Self {
            number_of_shards,
            number_of_modules: 0,
            number_of_initial_shards: number_of_shards,
            hashes: vec![],
            last_term_finished_block_num: 0,
            current_term_id: 0,
            seq: 0,
            params,
            term_params: params,
        }
    }

    pub fn number_of_shards(&self) -> &ShardId {
        &self.number_of_shards
    }

    pub fn number_of_modules(&self) -> &StorageId {
        &self.number_of_modules
    }

    pub fn add_shard(&mut self, tx_hash: TxHash) -> ShardId {
        let r = self.number_of_shards;
        self.number_of_shards += 1;
        self.hashes.push(tx_hash);
        r
    }

    pub fn add_module(&mut self) -> StorageId {
        let r = self.number_of_modules;
        self.number_of_modules += 1;
        r
    }

    #[cfg(test)]
    pub fn set_number_of_shards(&mut self, number_of_shards: ShardId) {
        assert!(self.number_of_shards <= number_of_shards);
        assert_eq!(0, self.hashes.len());
        self.number_of_shards = number_of_shards;
        self.number_of_initial_shards = number_of_shards;
    }

    pub fn shard_id_by_hash(&self, tx_hash: &TxHash) -> Option<ShardId> {
        debug_assert_eq!(::std::mem::size_of::<u16>(), ::std::mem::size_of::<::ctypes::ShardId>());
        assert!(self.hashes.len() < ::std::u16::MAX as usize);
        self.hashes.iter().enumerate().find(|(_index, hash)| tx_hash == *hash).map(|(index, _)| {
            let index = index as ShardId + self.number_of_initial_shards;
            assert!(index < self.number_of_shards);
            index
        })
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

    pub fn term_params(&self) -> &CommonParams {
        &self.term_params
    }

    pub fn update_term_params(&mut self) {
        self.term_params = self.params;
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
        self.number_of_shards == 0
    }
}

const PREFIX: u8 = super::Prefix::Metadata as u8;

impl Encodable for Metadata {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(10)
            .append(&PREFIX)
            .append(&self.number_of_shards)
            .append(&self.number_of_modules)
            .append(&self.number_of_initial_shards)
            .append_list(&self.hashes)
            .append(&self.last_term_finished_block_num)
            .append(&self.current_term_id)
            .append(&self.seq)
            .append(&self.params)
            .append(&self.term_params);
    }
}

impl Decodable for Metadata {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 10 {
            return Err(DecoderError::RlpInvalidLength {
                got: item_count,
                expected: 10,
            })
        }

        let prefix = rlp.val_at::<u8>(0)?;
        if PREFIX != prefix {
            cdebug!(STATE, "{} is not an expected prefix for metadata", prefix);
            return Err(DecoderError::Custom("Unexpected prefix"))
        }
        let number_of_shards = rlp.val_at(1)?;
        let number_of_modules = rlp.val_at(2)?;
        let number_of_initial_shards = rlp.val_at(3)?;
        let hashes = rlp.list_at(4)?;

        let last_term_finished_block_num = rlp.val_at(5)?;
        let current_term_id = rlp.val_at(6)?;
        let seq = rlp.val_at(7)?;
        let params = rlp.val_at(8)?;

        let term_params = rlp.val_at(9)?;

        Ok(Self {
            number_of_shards,
            number_of_modules,
            number_of_initial_shards,
            hashes,
            last_term_finished_block_num,
            current_term_id,
            seq,
            params,
            term_params,
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
            number_of_shards: 10,
            number_of_modules: 7,
            number_of_initial_shards: 1,
            hashes: vec![],
            last_term_finished_block_num: 0,
            current_term_id: 0,
            seq: 3,
            params: CommonParams::default_for_test(),
            term_params: CommonParams::default_for_test(),
        };
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn metadata_with_term_without_seq() {
        let metadata = Metadata {
            number_of_shards: 10,
            number_of_modules: 7,
            number_of_initial_shards: 1,
            hashes: vec![],
            last_term_finished_block_num: 0,
            current_term_id: 0,
            seq: 0,
            params: CommonParams::default_for_test(),
            term_params: CommonParams::default_for_test(),
        };
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn metadata_with_term_and_seq() {
        let metadata = Metadata {
            number_of_shards: 10,
            number_of_modules: 7,
            number_of_initial_shards: 1,
            hashes: vec![],
            last_term_finished_block_num: 1,
            current_term_id: 100,
            seq: 3,
            params: CommonParams::default_for_test(),
            term_params: CommonParams::default_for_test(),
        };
        rlp_encode_and_decode_test!(metadata);
    }
}
