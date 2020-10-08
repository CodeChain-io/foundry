// Copyright 2018-2020 Kodebox, Inc.
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

use crate::CacheableItem;
use ctypes::{ChainParams, StorageId};
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metadata {
    number_of_modules: StorageId,
    chain_params: ChainParams,
}

impl Metadata {
    pub fn new(chain_params: ChainParams) -> Self {
        Self {
            number_of_modules: 0,
            chain_params,
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

    pub fn chain_params(&self) -> &ChainParams {
        &self.chain_params
    }

    pub fn set_chain_params(&mut self, chain_params: ChainParams) {
        self.chain_params = chain_params;
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
        s.begin_list(3).append(&PREFIX).append(&self.number_of_modules).append(&self.chain_params);
    }
}

impl Decodable for Metadata {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 3 {
            return Err(DecoderError::RlpInvalidLength {
                got: item_count,
                expected: 3,
            })
        }

        let prefix = rlp.val_at::<u8>(0)?;
        if PREFIX != prefix {
            cdebug!(STATE, "{} is not an expected prefix for metadata", prefix);
            return Err(DecoderError::Custom("Unexpected prefix"))
        }
        let number_of_modules = rlp.val_at(1)?;
        let chain_params = rlp.val_at(2)?;

        Ok(Self {
            number_of_modules,
            chain_params,
        })
    }
}

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MetadataAddress(H256);

impl_address!(TOP, MetadataAddress, PREFIX);

impl MetadataAddress {
    pub fn new() -> Self {
        let mut key = [0u8; 32];
        key[0..16].copy_from_slice(b"metadata address");
        Self::from_transaction_hash(H256::from_slice(&key), 0)
    }
}

#[cfg(test)]
mod tests {
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
            hash.as_mut()[0] = PREFIX;
            hash
        };
        let address = MetadataAddress::from_hash(hash);
        assert_eq!(Some(MetadataAddress(hash)), address);
    }

    #[test]
    fn metadata() {
        let metadata = Metadata {
            number_of_modules: 7,
            chain_params: ChainParams::default_for_test(),
        };
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn metadata_address_is_constant() {
        let address = MetadataAddress::new();
        let expected = [
            77u8, 65, 87, 171, 93, 124, 219, 195, 206, 204, 98, 127, 21, 167, 126, 51, 136, 47, 48, 19, 43, 129, 199,
            232, 2, 92, 210, 163, 14, 209, 173, 33,
        ];
        assert_eq!(expected, address.as_ref());
    }
}
