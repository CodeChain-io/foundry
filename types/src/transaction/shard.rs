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

use super::{HashingError, PartialHashing};
use crate::util::tag::Tag;
use crate::{ShardId, Tracker, TxHash};
use ccrypto::blake256;
use ckey::NetworkId;
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

/// Shard Transaction type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShardTransaction {
    WrapCCC {
        network_id: NetworkId,
        shard_id: ShardId,
        tx_hash: TxHash,
    },
    ShardStore {
        network_id: NetworkId,
        shard_id: ShardId,
        content: String,
    },
}

impl ShardTransaction {
    pub fn tracker(&self) -> Tracker {
        if let ShardTransaction::WrapCCC {
            tx_hash,
            ..
        } = self
        {
            return (**tx_hash).into()
        }
        blake256(&*self.rlp_bytes()).into()
    }

    pub fn network_id(&self) -> NetworkId {
        match self {
            ShardTransaction::WrapCCC {
                network_id,
                ..
            }
            | ShardTransaction::ShardStore {
                network_id,
                ..
            } => *network_id,
        }
    }

    pub fn related_shards(&self) -> Vec<ShardId> {
        match self {
            ShardTransaction::ShardStore {
                shard_id,
                ..
            } => vec![*shard_id],
            ShardTransaction::WrapCCC {
                ..
            } => panic!("To be removed"),
        }
    }

    fn is_valid_output_index(&self, index: usize) -> bool {
        match self {
            ShardTransaction::WrapCCC {
                ..
            } => index == 0,
            ShardTransaction::ShardStore {
                ..
            } => index == 0,
        }
    }

    pub fn is_valid_shard_id_index(&self, index: usize, id: ShardId) -> bool {
        if !self.is_valid_output_index(index) {
            return false
        }
        match self {
            ShardTransaction::WrapCCC {
                shard_id,
                ..
            } => &id == shard_id,
            ShardTransaction::ShardStore {
                shard_id,
                ..
            } => &id == shard_id,
        }
    }
}

impl PartialHashing for ShardTransaction {
    fn hash_partially(&self, _tag: Tag, _is_burn: bool) -> Result<H256, HashingError> {
        // FIXME: delete this function
        Ok(Default::default())
    }
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum AssetID {
    /// Deprecated
    // COMPOSE_ID = 0x16,
    /// Deprecated
    // DECOMPOSE_ID = 0x17,
    ShardStore = 0x19,
}

impl Encodable for AssetID {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(*self as u8));
    }
}

impl Decodable for AssetID {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let tag = rlp.as_val()?;
        match tag {
            0x19u8 => Ok(AssetID::ShardStore),
            _ => Err(DecoderError::Custom("Unexpected AssetID Value")),
        }
    }
}

impl Decodable for ShardTransaction {
    fn decode(d: &Rlp<'_>) -> Result<Self, DecoderError> {
        match d.val_at(0)? {
            AssetID::ShardStore => {
                let item_count = d.item_count()?;
                if item_count != 4 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 4,
                    })
                }
                Ok(ShardTransaction::ShardStore {
                    network_id: d.val_at(1)?,
                    shard_id: d.val_at(2)?,
                    content: d.val_at(3)?,
                })
            }
        }
    }
}

impl Encodable for ShardTransaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            ShardTransaction::WrapCCC {
                ..
            } => {
                unreachable!("No reason to get a RLP encoding of WrapCCC");
            }
            ShardTransaction::ShardStore {
                network_id,
                shard_id,
                content,
            } => {
                s.begin_list(4).append(&AssetID::ShardStore).append(network_id).append(shard_id).append(content);
            }
        };
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn encode_and_decode_shard_store_text() {
        let tx = ShardTransaction::ShardStore {
            network_id: Default::default(),
            shard_id: 0,
            content: "content".to_string(),
        };
        rlp_encode_and_decode_test!(tx);
    }
}
