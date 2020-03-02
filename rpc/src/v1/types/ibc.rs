// Copyright 2019-2020 Kodebox, Inc.
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

use codechain_core::ibc::client_02::types::{ClientState as CoreClientState, ConsensusState as CoreConsensusState};
use codechain_core::ibc::connection_03::types::{
    ConnectionEnd as CoreConnectionEnd, ConnectionState as CoreConnectionState,
};
use primitives::H256;
use serde::Serialize;

type Identifier = String;
type CommitmentPrefix = String;

/// Many of RPC responses will be expressed with this
/// Because of the nature of IBC, they commonly
/// 1. Requires a block number for which proof stands
/// 2. The data should be transparent:
/// relayer must be able to open it and extract required infomation
/// 3. Includes a cryptographical proof of that
/// Note : proof may represents both that of presence and absence. It depends on option of data.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IBCQuery<T: Serialize> {
    pub number: u64,
    pub data: Option<T>,
    pub proof: String,
}

/// Client 02 related types

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientState {
    /// Unpacked light_client::ClientState
    pub number: u64,
    pub next_validator_set_hash: H256,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ConsensusState {
    pub validator_set_hash: H256,
    /// Unpacked CommitmentRoot
    pub state_root: H256,
}

impl ClientState {
    pub fn from_core(state: &CoreClientState) -> Self {
        ClientState {
            number: state.raw.number,
            next_validator_set_hash: state.raw.next_validator_set_hash,
        }
    }
}

impl ConsensusState {
    pub fn from_core(state: &CoreConsensusState) -> Self {
        ConsensusState {
            validator_set_hash: state.validator_set_hash,
            state_root: state.state_root.raw,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum ConnectionState {
    INIT,
    TRYOPEN,
    OPEN,
}

impl ConnectionState {
    pub fn from_core(core_connection_state: CoreConnectionState) -> Self {
        match core_connection_state {
            CoreConnectionState::INIT => ConnectionState::INIT,
            CoreConnectionState::TRYOPEN => ConnectionState::TRYOPEN,
            CoreConnectionState::OPEN => ConnectionState::OPEN,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionEnd {
    pub state: ConnectionState,
    pub counterparty_connection_identifier: Identifier,
    pub counterparty_prefix: CommitmentPrefix,
    pub client_identifier: Identifier,
    pub counterparty_client_identifier: Identifier,
}

impl ConnectionEnd {
    pub fn from_core(core_connection_end: CoreConnectionEnd) -> Self {
        ConnectionEnd {
            state: ConnectionState::from_core(core_connection_end.state),
            counterparty_connection_identifier: core_connection_end.counterparty_connection_identifier,
            counterparty_prefix: core_connection_end.counterparty_prefix.raw,
            client_identifier: core_connection_end.client_identifier,
            counterparty_client_identifier: core_connection_end.counterparty_client_identifier,
        }
    }
}

pub type ConnectionIdentifiersInClient = Vec<String>;
