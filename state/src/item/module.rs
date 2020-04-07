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
use ccrypto::BLAKE_NULL_RLP;
use ctypes::StorageId;
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Debug)]
pub struct Module {
    root: H256,
}

impl Module {
    pub fn new(module_root: H256) -> Self {
        Self {
            root: module_root,
        }
    }

    pub fn root(&self) -> &H256 {
        &self.root
    }

    pub fn set_root(&mut self, root: H256) {
        self.root = root;
    }
}

impl Default for Module {
    fn default() -> Self {
        Self::new(BLAKE_NULL_RLP)
    }
}

impl CacheableItem for Module {
    type Address = ModuleAddress;

    fn is_null(&self) -> bool {
        self.root == BLAKE_NULL_RLP
    }
}

const PREFIX: u8 = super::Prefix::Module as u8;

impl Encodable for Module {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2).append(&PREFIX).append(&self.root);
    }
}

impl Decodable for Module {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 2 {
            return Err(DecoderError::RlpInvalidLength {
                expected: 2,
                got: item_count,
            })
        }
        let prefix = rlp.val_at::<u8>(0)?;
        if PREFIX != prefix {
            cdebug!(STATE, "{} is not an expected prefix for module", prefix);
            return Err(DecoderError::Custom("Unexpected prefix"))
        }
        Ok(Self {
            root: rlp.val_at(1)?,
        })
    }
}

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ModuleAddress(H256);

impl_address!(TOP, ModuleAddress, PREFIX);

impl ModuleAddress {
    pub fn new(storage_id: StorageId) -> Self {
        Self::from_transaction_hash(H256::from_slice(b"module"), storage_id.into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn different_storage_id_makes_different_address() {
        let address1 = ModuleAddress::new(0);
        let address2 = ModuleAddress::new(1);
        assert_ne!(address1, address2);
        assert_eq!(address1[0], PREFIX);
        assert_eq!(address2[0], PREFIX);
    }

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
        let address = ModuleAddress::from_hash(hash);
        assert!(address.is_none());
    }

    #[test]
    fn parse_return_some() {
        let hash = {
            let mut hash = H256::random();
            hash[0] = PREFIX;
            hash
        };
        let address = ModuleAddress::from_hash(hash);
        assert_eq!(Some(ModuleAddress(hash)), address);
    }
}
