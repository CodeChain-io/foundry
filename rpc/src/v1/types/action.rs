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
use primitives::Bytes;
use rlp::Encodable;
use std::convert::TryFrom;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase", tag = "type")]
pub enum Action {
    Pay {
        receiver: PlatformAddress,
        quantity: Uint,
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
    UpdateValidators,
    CloseTerm,
    ChangeNextValidators,
    Elect,
}

impl Action {
    pub fn from_core(from: ActionType, network_id: NetworkId) -> Self {
        match from {
            ActionType::Pay {
                receiver,
                quantity,
            } => Action::Pay {
                receiver: PlatformAddress::new_v0(network_id, receiver),
                quantity: quantity.into(),
            },
            ActionType::TransferCCS {
                address,
                quantity,
            } => Action::TransferCCS {
                address: PlatformAddress::new_v0(network_id, address),
                quantity: quantity.into(),
            },
            ActionType::DelegateCCS {
                address,
                quantity,
            } => Action::DelegateCCS {
                address: PlatformAddress::new_v0(network_id, address),
                quantity: quantity.into(),
            },
            ActionType::Revoke {
                address,
                quantity,
            } => Action::Revoke {
                address: PlatformAddress::new_v0(network_id, address),
                quantity: quantity.into(),
            },
            ActionType::Redelegate {
                prev_delegatee,
                next_delegatee,
                quantity,
            } => Action::Redelegate {
                prev_delegatee: PlatformAddress::new_v0(network_id, prev_delegatee),
                next_delegatee: PlatformAddress::new_v0(network_id, next_delegatee),
                quantity: quantity.into(),
            },
            ActionType::SelfNominate {
                deposit,
                metadata,
            } => Action::SelfNominate {
                deposit: deposit.into(),
                metadata,
            },
            ActionType::ChangeParams {
                metadata_seq,
                params,
                approvals,
            } => Action::ChangeParams {
                metadata_seq: metadata_seq.into(),
                params: params.rlp_bytes(),
                approvals,
            },
            ActionType::ReportDoubleVote {
                message1,
                message2,
            } => Action::ReportDoubleVote {
                message1,
                message2,
            },
            ActionType::UpdateValidators {
                ..
            } => Action::UpdateValidators, // TODO: Implement serialization
            ActionType::CloseTerm {
                ..
            } => Action::CloseTerm, // TODO: Implement serialization
            ActionType::ChangeNextValidators {
                ..
            } => Action::ChangeNextValidators, // TODO: Implement serialization
            ActionType::Elect {
                ..
            } => Action::Elect, // TODO: Implement serialization
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
                receiver: receiver.try_into_pubkey()?,
                quantity: quantity.into(),
            },
            Action::TransferCCS {
                address,
                quantity,
            } => ActionType::TransferCCS {
                address: address.try_into_pubkey()?,
                quantity: quantity.into(),
            },
            Action::DelegateCCS {
                address,
                quantity,
            } => ActionType::DelegateCCS {
                address: address.try_into_pubkey()?,
                quantity: quantity.into(),
            },
            Action::Revoke {
                address,
                quantity,
            } => ActionType::Revoke {
                address: address.try_into_pubkey()?,
                quantity: quantity.into(),
            },
            Action::Redelegate {
                prev_delegatee,
                next_delegatee,
                quantity,
            } => ActionType::Redelegate {
                prev_delegatee: prev_delegatee.try_into_pubkey()?,
                next_delegatee: next_delegatee.try_into_pubkey()?,
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
                params: Box::new(rlp::decode(&params).map_err(KeyError::from)?),
                approvals,
            },
            Action::ReportDoubleVote {
                message1,
                message2,
            } => ActionType::ReportDoubleVote {
                message1,
                message2,
            },
            Action::UpdateValidators => unreachable!("No reason to get UpdateValidators from RPCs"),
            Action::CloseTerm => unreachable!("No reason to get CloseTerm from RPCs"),
            Action::ChangeNextValidators => unreachable!("No reason to get ChangeNextValidators from RPCs"),
            Action::Elect => unreachable!("No reason to get Elect from RPCs"),
        })
    }
}
