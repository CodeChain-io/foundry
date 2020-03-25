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
use ckey::{NetworkId, PlatformAddress};
use ctypes::transaction::Action as ActionType;
use primitives::Bytes;
use std::convert::TryFrom;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Action {
    Pay {
        receiver: PlatformAddress,
        quantity: Uint,
    },
    #[serde(rename_all = "camelCase")]
    Custom {
        handler_id: Uint,
        bytes: Bytes,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum ActionWithTracker {
    Pay {
        receiver: PlatformAddress,
        quantity: Uint,
    },
    #[serde(rename_all = "camelCase")]
    Custom {
        handler_id: Uint,
        bytes: Bytes,
    },
}

impl ActionWithTracker {
    pub fn from_core(from: ActionType, network_id: NetworkId) -> Self {
        match from {
            ActionType::Pay {
                receiver,
                quantity,
            } => ActionWithTracker::Pay {
                receiver: PlatformAddress::new_v1(network_id, receiver),
                quantity: quantity.into(),
            },
            ActionType::Custom {
                handler_id,
                bytes,
            } => ActionWithTracker::Custom {
                handler_id: handler_id.into(),
                bytes,
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
            Action::Custom {
                handler_id,
                bytes,
            } => ActionType::Custom {
                handler_id: handler_id.into(),
                bytes,
            },
        })
    }
}
