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

use crate::uint::Uint;
use ckey::{Ed25519Public as Public, PlatformAddress};
use std::collections::HashMap;

/// Tendermint params deserialization.
#[derive(Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TendermintParams {
    /// Propose step timeout in milliseconds.
    pub timeout_propose: Option<Uint>,
    /// Propose step timeout delta in milliseconds.
    pub timeout_propose_delta: Option<Uint>,
    /// Prevote step timeout in milliseconds.
    pub timeout_prevote: Option<Uint>,
    /// Prevote step timeout delta in milliseconds.
    pub timeout_prevote_delta: Option<Uint>,
    /// Precommit step timeout in milliseconds.
    pub timeout_precommit: Option<Uint>,
    /// Precommit step timeout delta in milliseconds.
    pub timeout_precommit_delta: Option<Uint>,
    /// Commit step timeout in milliseconds.
    pub timeout_commit: Option<Uint>,
    /// allowed past time gap in milliseconds.
    pub allowed_past_timegap: Option<Uint>,
    /// allowed future time gap in milliseconds.
    pub allowed_future_timegap: Option<Uint>,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Deposit {
    pub pubkey: Public,
    pub deposit: u64,
    pub nomination_ends_at: u64,
    pub metadata: String,
}

#[derive(Clone, Debug, PartialEq, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StakeAccount {
    pub stake: u64,
    pub delegations: Option<HashMap<PlatformAddress, u64>>,
}

/// Tendermint engine deserialization.
#[derive(Debug, PartialEq, Deserialize)]
pub struct Tendermint {
    pub params: TendermintParams,
}
