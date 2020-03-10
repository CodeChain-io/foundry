// Copyright 2020 Kodebox, Inc.
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
use ccrypto::Blake;
use ctypes::StorageId;
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

/// Module level datum in the DB.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ModuleDatum {
    // Content of the datum
    datum: Vec<u8>,
}

impl ModuleDatum {
    pub fn new(datum: Vec<u8>) -> Self {
        Self {
            datum,
        }
    }

    /// Get clone of the datum
    pub fn content(&self) -> Vec<u8> {
        self.datum.clone()
    }

    /// Get blake hash of the content of the text
    pub fn content_hash(&self) -> H256 {
        let rlp = self.datum.rlp_bytes();
        Blake::blake(rlp)
    }
}

const PREFIX: u8 = super::Prefix::ModuleDatum as u8;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleDatumAddress(H256);

impl_address!(MODULE, ModuleDatumAddress, PREFIX);

impl ModuleDatumAddress {
    pub fn new<T: AsRef<[u8]>>(key: T, storage_id: StorageId) -> Self {
        Self::from_key_with_storage_id(key, storage_id)
    }
}

impl CacheableItem for ModuleDatum {
    type Address = ModuleDatumAddress;
    /// Check if content is empty and certifier is null.
    fn is_null(&self) -> bool {
        self.datum.is_empty()
    }
}

impl Encodable for ModuleDatum {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2);
        s.append(&PREFIX);
        s.append(&self.datum);
    }
}

impl Decodable for ModuleDatum {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 2 {
            return Err(DecoderError::RlpInvalidLength {
                got: item_count,
                expected: 2,
            })
        }
        let prefix = rlp.val_at::<u8>(0)?;
        if PREFIX != prefix {
            cdebug!(STATE, "{} is not an expected prefix for module datum", prefix);
            return Err(DecoderError::Custom("Unexpected prefix"))
        }
        Ok(Self {
            datum: rlp.val_at(1)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn rlp_encode_and_decode() {
        rlp_encode_and_decode_test!(ModuleDatum {
            datum: String::from("Foundry").into_bytes(),
        });
    }

    #[test]
    fn cachable_item_is_null() {
        let datum: ModuleDatum = Default::default();
        assert!(datum.is_null());
    }

    #[test]
    fn address_prefix() {
        let address = ModuleDatumAddress::new("Foundry", 500);
        let ModuleDatumAddress(address) = address;
        let module_prefix = [83, 0]; // ord('S') == 83
        let storage_prefix = [1, 244]; // hex(500) == 0x1f4;
        assert_eq!(&address[0..2], &module_prefix);
        assert_eq!(&address[2..4], &storage_prefix);
    }

    #[test]
    fn different_storage_id_makes_different_address() {
        let address1 = ModuleDatumAddress::new("foundry", 1);
        let address2 = ModuleDatumAddress::new("foundry", 2);
        assert_ne!(address1, address2);
        assert_eq!(address1[0], PREFIX);
        assert_eq!(address2[0], PREFIX);
    }
}
