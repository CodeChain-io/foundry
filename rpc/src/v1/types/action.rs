// Copyright 2018-2019 Kodebox, Inc.
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

use std::convert::TryFrom;

use cjson::uint::Uint;
use ckey::{NetworkId, PlatformAddress, Public, Signature};
use ctypes::transaction::Action as ActionType;
use ctypes::TxHash;
use primitives::Bytes;

use super::super::errors::ConversionError;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Action {
    Pay {
        receiver: PlatformAddress,
        quantity: Uint,
    },
    SetRegularKey {
        key: Public,
    },
    Store {
        content: String,
        certifier: PlatformAddress,
        signature: Signature,
    },
    Remove {
        hash: TxHash,
        signature: Signature,
    },
    #[serde(rename_all = "camelCase")]
    Custom {
        handler_id: Uint,
        bytes: Bytes,
    },
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
            Action::Store {
                content,
                certifier,
                signature,
            } => ActionType::Store {
                content,
                certifier: certifier.try_into_address()?,
                signature,
            },
            Action::Remove {
                hash,
                signature,
            } => ActionType::Remove {
                hash,
                signature,
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

impl Action {
    pub fn from_core(from: ActionType, network_id: NetworkId) -> Self {
        match from {
            ActionType::Pay {
                receiver,
                quantity,
            } => Action::Pay {
                receiver: PlatformAddress::new_v1(network_id, receiver),
                quantity: quantity.into(),
            },
            ActionType::SetRegularKey {
                key,
            } => Action::SetRegularKey {
                key,
            },
            ActionType::Store {
                content,
                certifier,
                signature,
            } => Action::Store {
                content,
                certifier: PlatformAddress::new_v1(network_id, certifier),
                signature,
            },
            ActionType::Remove {
                hash,
                signature,
            } => Action::Remove {
                hash,
                signature,
            },
            ActionType::Custom {
                handler_id,
                bytes,
            } => Action::Custom {
                handler_id: handler_id.into(),
                bytes,
            },
        }
    }
}
