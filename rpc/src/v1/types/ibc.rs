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
    ConnectionEnd as CoreConnectionEnd, ConnectionIdentifiersInClient as CoreConnectionIdentifiersInClient,
    ConnectionState as CoreConnectionState,
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

pub trait FromCore<T> {
    fn from_core(core: T) -> Self;
}

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

impl FromCore<CoreClientState> for ClientState {
    fn from_core(core: CoreClientState) -> Self {
        ClientState {
            number: core.raw.number,
            next_validator_set_hash: core.raw.next_validator_set_hash,
        }
    }
}

impl FromCore<CoreConsensusState> for ConsensusState {
    fn from_core(core: CoreConsensusState) -> Self {
        ConsensusState {
            validator_set_hash: core.validator_set_hash,
            state_root: core.state_root.raw,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum ConnectionState {
    INIT,
    TRYOPEN,
    OPEN,
}

impl FromCore<CoreConnectionState> for ConnectionState {
    fn from_core(core: CoreConnectionState) -> Self {
        match core {
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

impl FromCore<CoreConnectionEnd> for ConnectionEnd {
    fn from_core(core: CoreConnectionEnd) -> Self {
        ConnectionEnd {
            state: ConnectionState::from_core(core.state),
            counterparty_connection_identifier: core.counterparty_connection_identifier,
            counterparty_prefix: core.counterparty_prefix.raw,
            client_identifier: core.client_identifier,
            counterparty_client_identifier: core.counterparty_client_identifier,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConnectionIdentifiersInClient {
    raw: Vec<String>,
}

impl FromCore<CoreConnectionIdentifiersInClient> for ConnectionIdentifiersInClient {
    fn from_core(core: CoreConnectionIdentifiersInClient) -> Self {
        ConnectionIdentifiersInClient {
            raw: core.into_vec(),
        }
    }
}
