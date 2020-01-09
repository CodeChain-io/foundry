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

use crate::errors::SyntaxError;
use crate::transaction::ShardTransaction;
use crate::{CommonParams, ShardId, Tracker};
use ccrypto::Blake;
use ckey::{recover, Address, NetworkId, Public, Signature};
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Copy)]
#[repr(u8)]
enum ActionTag {
    Pay = 0x02,
    SetRegularKey = 0x03,
    CreateShard = 0x04,
    SetShardOwners = 0x05,
    SetShardUsers = 0x06,
    ShardStore = 0x19,
    Custom = 0xFF,
}

impl Encodable for ActionTag {
    fn rlp_append(&self, s: &mut RlpStream) {
        (*self as u8).rlp_append(s)
    }
}

impl Decodable for ActionTag {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let tag = rlp.as_val()?;
        match tag {
            0x02u8 => Ok(Self::Pay),
            0x03 => Ok(Self::SetRegularKey),
            0x04 => Ok(Self::CreateShard),
            0x05 => Ok(Self::SetShardOwners),
            0x06 => Ok(Self::SetShardUsers),
            0x19 => Ok(Self::ShardStore),
            0xFF => Ok(Self::Custom),
            _ => Err(DecoderError::Custom("Unexpected action prefix")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Pay {
        receiver: Address,
        /// Transferred quantity.
        quantity: u64,
    },
    SetRegularKey {
        key: Public,
    },
    CreateShard {
        users: Vec<Address>,
    },
    SetShardOwners {
        shard_id: ShardId,
        owners: Vec<Address>,
    },
    SetShardUsers {
        shard_id: ShardId,
        users: Vec<Address>,
    },
    Custom {
        handler_id: u64,
        bytes: Bytes,
    },
    ShardStore {
        network_id: NetworkId,
        shard_id: ShardId,
        content: String,
    },
}

impl Action {
    pub fn hash(&self) -> H256 {
        let rlp = self.rlp_bytes();
        Blake::blake(rlp)
    }

    pub fn shard_transaction(&self) -> Option<ShardTransaction> {
        match self {
            Action::ShardStore {
                ..
            } => self.clone().into(),
            _ => None,
        }
    }

    pub fn tracker(&self) -> Option<Tracker> {
        self.shard_transaction().map(|tx| tx.tracker())
    }

    pub fn verify(&self) -> Result<(), SyntaxError> {
        Ok(())
    }

    pub fn verify_with_params(&self, common_params: &CommonParams) -> Result<(), SyntaxError> {
        if let Some(network_id) = self.network_id() {
            let system_network_id = common_params.network_id();
            if network_id != system_network_id {
                return Err(SyntaxError::InvalidNetworkId(network_id))
            }
        }

        Ok(())
    }

    // FIXME: We don't use signer any more
    pub fn verify_with_signer_address(&self, _signer: &Address) -> Result<(), SyntaxError> {
        if let Some(approvals) = self.approvals() {
            let tracker = self.tracker().unwrap();

            for approval in approvals {
                recover(approval, &tracker).map_err(|err| SyntaxError::InvalidApproval(err.to_string()))?;
            }
        }
        Ok(())
    }

    // FIXME: please remove this function
    fn approvals(&self) -> Option<&[Signature]> {
        None
    }

    fn network_id(&self) -> Option<NetworkId> {
        match self {
            Action::ShardStore {
                network_id,
                ..
            } => Some(*network_id),
            _ => None,
        }
    }
}

impl From<Action> for Option<ShardTransaction> {
    fn from(action: Action) -> Self {
        match action {
            Action::ShardStore {
                network_id,
                shard_id,
                content,
            } => Some(ShardTransaction::ShardStore {
                network_id,
                shard_id,
                content,
            }),
            _ => None,
        }
    }
}

impl Encodable for Action {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            Action::Pay {
                receiver,
                quantity,
            } => {
                s.begin_list(3);
                s.append(&ActionTag::Pay);
                s.append(receiver);
                s.append(quantity);
            }
            Action::SetRegularKey {
                key,
            } => {
                s.begin_list(2);
                s.append(&ActionTag::SetRegularKey);
                s.append(key);
            }
            Action::CreateShard {
                users,
            } => {
                s.begin_list(2);
                s.append(&ActionTag::CreateShard);
                s.append_list(users);
            }
            Action::SetShardOwners {
                shard_id,
                owners,
            } => {
                s.begin_list(3);
                s.append(&ActionTag::SetShardOwners);
                s.append(shard_id);
                s.append_list(owners);
            }
            Action::SetShardUsers {
                shard_id,
                users,
            } => {
                s.begin_list(3);
                s.append(&ActionTag::SetShardUsers);
                s.append(shard_id);
                s.append_list(users);
            }
            Action::Custom {
                handler_id,
                bytes,
            } => {
                s.begin_list(3);
                s.append(&ActionTag::Custom);
                s.append(handler_id);
                s.append(bytes);
            }
            Action::ShardStore {
                shard_id,
                content,
                network_id,
            } => {
                s.begin_list(4);
                s.append(&ActionTag::ShardStore);
                s.append(network_id);
                s.append(shard_id);
                s.append(content);
            }
        }
    }
}

impl Decodable for Action {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        match rlp.val_at(0)? {
            ActionTag::Pay => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 3,
                    })
                }
                Ok(Action::Pay {
                    receiver: rlp.val_at(1)?,
                    quantity: rlp.val_at(2)?,
                })
            }
            ActionTag::SetRegularKey => {
                let item_count = rlp.item_count()?;
                if item_count != 2 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 2,
                    })
                }
                Ok(Action::SetRegularKey {
                    key: rlp.val_at(1)?,
                })
            }
            ActionTag::CreateShard => {
                let item_count = rlp.item_count()?;
                if item_count != 2 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 2,
                    })
                }
                Ok(Action::CreateShard {
                    users: rlp.list_at(1)?,
                })
            }
            ActionTag::SetShardOwners => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 3,
                    })
                }
                Ok(Action::SetShardOwners {
                    shard_id: rlp.val_at(1)?,
                    owners: rlp.list_at(2)?,
                })
            }
            ActionTag::SetShardUsers => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 3,
                    })
                }
                Ok(Action::SetShardUsers {
                    shard_id: rlp.val_at(1)?,
                    users: rlp.list_at(2)?,
                })
            }
            ActionTag::Custom => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 3,
                    })
                }
                Ok(Action::Custom {
                    handler_id: rlp.val_at(1)?,
                    bytes: rlp.val_at(2)?,
                })
            }
            ActionTag::ShardStore => {
                let item_count = rlp.item_count()?;
                if item_count != 4 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 4,
                    })
                }
                Ok(Action::ShardStore {
                    network_id: rlp.val_at(1)?,
                    shard_id: rlp.val_at(2)?,
                    content: rlp.val_at(3)?,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn encode_and_decode_pay_action() {
        rlp_encode_and_decode_test!(Action::Pay {
            receiver: Address::random(),
            quantity: 300,
        });
    }

    #[test]
    fn encode_and_decode_set_shard_owners() {
        rlp_encode_and_decode_test!(Action::SetShardOwners {
            shard_id: 1,
            owners: vec![Address::random(), Address::random()],
        });
    }

    #[test]
    fn encode_and_decode_set_shard_users() {
        rlp_encode_and_decode_test!(Action::SetShardUsers {
            shard_id: 1,
            users: vec![Address::random(), Address::random()],
        });
    }
}
