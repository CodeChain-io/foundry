// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use ccrypto::Blake;
use ctypes::{ShardId, Tracker};
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use crate::CacheableItem;

/// Text stored in the DB. Used by Store/Remove Action.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ShardText {
    // Content of the text
    content: String,
}

impl ShardText {
    pub fn new(content: &str) -> Self {
        Self {
            content: content.to_string(),
        }
    }

    /// Get reference of the content of the text
    pub fn content(&self) -> &String {
        &self.content
    }

    /// Get blake hash of the content of the text
    pub fn content_hash(&self) -> H256 {
        let rlp = self.content.rlp_bytes();
        Blake::blake(rlp)
    }
}

const PREFIX: u8 = super::SHARD_TEXT_PREFIX;

#[derive(Clone, Copy, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShardTextAddress(H256);

impl_address!(SHARD, ShardTextAddress, PREFIX);

impl ShardTextAddress {
    pub fn new(tracker: Tracker, shard_id: ShardId) -> Self {
        let index = ::std::u64::MAX;

        Self::from_hash_with_shard_id(*tracker, index, shard_id)
    }
}

impl CacheableItem for ShardText {
    type Address = ShardTextAddress;
    /// Check if content is empty and certifier is null.
    fn is_null(&self) -> bool {
        self.content.is_empty()
    }
}

impl Encodable for ShardText {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2);
        s.append(&PREFIX);
        s.append(&self.content);
    }
}

impl Decodable for ShardText {
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
            cdebug!(STATE, "{} is not an expected prefix for asset", prefix);
            return Err(DecoderError::Custom("Unexpected prefix"))
        }
        Ok(Self {
            content: rlp.val_at(1)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn rlp_encode_and_decode() {
        rlp_encode_and_decode_test!(ShardText {
            content: "CodeChain".to_string(),
        });
    }

    #[test]
    fn cachable_item_is_null() {
        let text: ShardText = Default::default();
        assert!(text.is_null());
    }
}
