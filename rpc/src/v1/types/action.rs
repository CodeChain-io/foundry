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

use super::super::errors::ConversionError;
use cjson::uint::Uint;
use ckey::{Error as KeyError, NetworkId, PlatformAddress};
use ctypes::transaction::{Action as ActionType, Approval};
use ctypes::{ShardId, Tracker};
use primitives::Bytes;
use rlp::Encodable;
use std::convert::TryFrom;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Action {
    Pay {
        receiver: PlatformAddress,
        quantity: Uint,
    },
    CreateShard {
        users: Vec<PlatformAddress>,
    },
    #[serde(rename_all = "camelCase")]
    SetShardOwners {
        shard_id: ShardId,
        owners: Vec<PlatformAddress>,
    },
    #[serde(rename_all = "camelCase")]
    SetShardUsers {
        shard_id: ShardId,
        users: Vec<PlatformAddress>,
    },
    ShardStore {
        network_id: NetworkId,
        shard_id: ShardId,
        content: String,
    },
    TransferCCS {
        address: PlatformAddress,
        quantity: Uint,
    },
    DelegateCCS {
        address: PlatformAddress,
        quantity: Uint,
    },
    Revoke {
        address: PlatformAddress,
        quantity: Uint,
    },
    #[serde(rename_all = "camelCase")]
    Redelegate {
        prev_delegatee: PlatformAddress,
        next_delegatee: PlatformAddress,
        quantity: Uint,
    },
    SelfNominate {
        deposit: Uint,
        metadata: Bytes,
    },
    #[serde(rename_all = "camelCase")]
    ChangeParams {
        metadata_seq: Uint,
        params: Bytes,
        approvals: Vec<Approval>,
    },
    ReportDoubleVote {
        message1: Bytes,
        message2: Bytes,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ActionWithTracker {
    Pay {
        receiver: PlatformAddress,
        quantity: Uint,
    },
    CreateShard {
        users: Vec<PlatformAddress>,
    },
    #[serde(rename_all = "camelCase")]
    SetShardOwners {
        shard_id: ShardId,
        owners: Vec<PlatformAddress>,
    },
    #[serde(rename_all = "camelCase")]
    SetShardUsers {
        shard_id: ShardId,
        users: Vec<PlatformAddress>,
    },
    ShardStore {
        network_id: NetworkId,
        shard_id: ShardId,
        content: String,
        tracker: Tracker,
    },
    TransferCCS {
        address: PlatformAddress,
        quantity: Uint,
    },
    DelegateCCS {
        address: PlatformAddress,
        quantity: Uint,
    },
    Revoke {
        address: PlatformAddress,
        quantity: Uint,
    },
    #[serde(rename_all = "camelCase")]
    Redelegate {
        prev_delegatee: PlatformAddress,
        next_delegatee: PlatformAddress,
        quantity: Uint,
    },
    SelfNominate {
        deposit: Uint,
        metadata: Bytes,
    },
    #[serde(rename_all = "camelCase")]
    ChangeParams {
        metadata_seq: Uint,
        params: Bytes,
        approvals: Vec<Approval>,
    },
    ReportDoubleVote {
        message1: Bytes,
        message2: Bytes,
    },
}

impl ActionWithTracker {
    pub fn from_core(from: ActionType, network_id: NetworkId) -> Self {
        let tracker = from.tracker();
        match from {
            ActionType::Pay {
                receiver,
                quantity,
            } => ActionWithTracker::Pay {
                receiver: PlatformAddress::new_v1(network_id, receiver),
                quantity: quantity.into(),
            },
            ActionType::CreateShard {
                users,
            } => {
                let users = users.into_iter().map(|user| PlatformAddress::new_v1(network_id, user)).collect();
                ActionWithTracker::CreateShard {
                    users,
                }
            }
            ActionType::SetShardOwners {
                shard_id,
                owners,
            } => ActionWithTracker::SetShardOwners {
                shard_id,
                owners: owners.into_iter().map(|owner| PlatformAddress::new_v1(network_id, owner)).collect(),
            },
            ActionType::SetShardUsers {
                shard_id,
                users,
            } => ActionWithTracker::SetShardUsers {
                shard_id,
                users: users.into_iter().map(|user| PlatformAddress::new_v1(network_id, user)).collect(),
            },
            ActionType::ShardStore {
                network_id,
                shard_id,
                content,
            } => ActionWithTracker::ShardStore {
                network_id,
                shard_id,
                content,
                tracker: tracker.unwrap(),
            },
            ActionType::TransferCCS {
                address,
                quantity,
            } => ActionWithTracker::TransferCCS {
                address: PlatformAddress::new_v1(network_id, address),
                quantity: quantity.into(),
            },
            ActionType::DelegateCCS {
                address,
                quantity,
            } => ActionWithTracker::DelegateCCS {
                address: PlatformAddress::new_v1(network_id, address),
                quantity: quantity.into(),
            },
            ActionType::Revoke {
                address,
                quantity,
            } => ActionWithTracker::Revoke {
                address: PlatformAddress::new_v1(network_id, address),
                quantity: quantity.into(),
            },
            ActionType::Redelegate {
                prev_delegatee,
                next_delegatee,
                quantity,
            } => ActionWithTracker::Redelegate {
                prev_delegatee: PlatformAddress::new_v1(network_id, prev_delegatee),
                next_delegatee: PlatformAddress::new_v1(network_id, next_delegatee),
                quantity: quantity.into(),
            },
            ActionType::SelfNominate {
                deposit,
                metadata,
            } => ActionWithTracker::SelfNominate {
                deposit: deposit.into(),
                metadata,
            },
            ActionType::ChangeParams {
                metadata_seq,
                params,
                approvals,
            } => ActionWithTracker::ChangeParams {
                metadata_seq: metadata_seq.into(),
                params: params.rlp_bytes(),
                approvals,
            },
            ActionType::ReportDoubleVote {
                message1,
                message2,
            } => ActionWithTracker::ReportDoubleVote {
                message1,
                message2,
            },
        }
    }
}

impl TryFrom<Action> for ActionType {
    type Error = ConversionError;
    fn try_from(from: Action) -> Result<Self, Self::Error> {
        Ok(match from {
            Action::Pay {
                receiver,
                quantity,
            } => ActionType::Pay {
                receiver: receiver.try_into_address()?,
                quantity: quantity.into(),
            },
            Action::CreateShard {
                users,
            } => {
                let users = users.into_iter().map(PlatformAddress::try_into_address).collect::<Result<_, _>>()?;
                ActionType::CreateShard {
                    users,
                }
            }
            Action::SetShardOwners {
                shard_id,
                owners,
            } => {
                let owners: Result<_, _> = owners.into_iter().map(PlatformAddress::try_into_address).collect();
                ActionType::SetShardOwners {
                    shard_id,
                    owners: owners?,
                }
            }
            Action::SetShardUsers {
                shard_id,
                users,
            } => {
                let users: Result<_, _> = users.into_iter().map(PlatformAddress::try_into_address).collect();
                ActionType::SetShardUsers {
                    shard_id,
                    users: users?,
                }
            }
            Action::ShardStore {
                network_id,
                shard_id,
                content,
            } => ActionType::ShardStore {
                network_id,
                shard_id,
                content,
            },
            Action::TransferCCS {
                address,
                quantity,
            } => ActionType::TransferCCS {
                address: address.try_into_address()?,
                quantity: quantity.into(),
            },
            Action::DelegateCCS {
                address,
                quantity,
            } => ActionType::DelegateCCS {
                address: address.try_into_address()?,
                quantity: quantity.into(),
            },
            Action::Revoke {
                address,
                quantity,
            } => ActionType::Revoke {
                address: address.try_into_address()?,
                quantity: quantity.into(),
            },
            Action::Redelegate {
                prev_delegatee,
                next_delegatee,
                quantity,
            } => ActionType::Redelegate {
                prev_delegatee: prev_delegatee.try_into_address()?,
                next_delegatee: next_delegatee.try_into_address()?,
                quantity: quantity.into(),
            },
            Action::SelfNominate {
                deposit,
                metadata,
            } => ActionType::SelfNominate {
                deposit: deposit.into(),
                metadata,
            },
            Action::ChangeParams {
                metadata_seq,
                params,
                approvals,
            } => ActionType::ChangeParams {
                metadata_seq: metadata_seq.into(),
                params: Box::new(rlp::decode(&params).map_err(|err| KeyError::Custom(format!("{:?}", err)))?),
                approvals,
            },
            Action::ReportDoubleVote {
                message1,
                message2,
            } => ActionType::ReportDoubleVote {
                message1,
                message2,
            },
        })
    }
}
