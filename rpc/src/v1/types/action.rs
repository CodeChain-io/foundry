// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
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
}

impl Action {
    pub fn from_core(from: ActionType, network_id: NetworkId) -> Self {
        Action
    }
}

impl TryFrom<Action> for ActionType {
    type Error = ConversionError;
    fn try_from(from: Action) -> Result<Self, Self::Error> {
        Ok(ActionType)
    }
}
