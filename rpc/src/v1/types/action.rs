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
use ckey::{NetworkId, PlatformAddress, Public};
use ctypes::transaction::Action as ActionType;
use ctypes::{ShardId, Tracker};
use primitives::Bytes;
use std::convert::TryFrom;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Action {
    Pay {
        receiver: PlatformAddress,
        quantity: Uint,
    },
    SetRegularKey {
        key: Public,
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
    #[serde(rename_all = "camelCase")]
    Custom {
        handler_id: Uint,
        bytes: Bytes,
    },
    ShardStore {
        network_id: NetworkId,
        shard_id: ShardId,
        content: String,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ActionWithTracker {
    Pay {
        receiver: PlatformAddress,
        quantity: Uint,
    },
    SetRegularKey {
        key: Public,
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
    #[serde(rename_all = "camelCase")]
    Custom {
        handler_id: Uint,
        bytes: Bytes,
    },
    ShardStore {
        network_id: NetworkId,
        shard_id: ShardId,
        content: String,
        tracker: Tracker,
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
            ActionType::SetRegularKey {
                key,
            } => ActionWithTracker::SetRegularKey {
                key,
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
            ActionType::Custom {
                handler_id,
                bytes,
            } => ActionWithTracker::Custom {
                handler_id: handler_id.into(),
                bytes,
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
            ActionType::TransferAsset {
                ..
            } => unimplemented!("To be removed"),
            ActionType::UnwrapCCC {
                ..
            } => unimplemented!("To be removed"),
            ActionType::WrapCCC {
                ..
            } => unimplemented!("To be removed"),
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
            Action::SetRegularKey {
                key,
            } => ActionType::SetRegularKey {
                key,
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
            Action::Custom {
                handler_id,
                bytes,
            } => ActionType::Custom {
                handler_id: handler_id.into(),
                bytes,
            },
            Action::ShardStore {
                network_id,
                shard_id,
                content,
            } => ActionType::ShardStore {
                network_id,
                shard_id,
                content,
            },
        })
    }
}
