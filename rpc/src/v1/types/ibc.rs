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

use codechain_core::ibc::channel_04::types::{
    AcknowledgementHash as CoreAcknowledgementHash, ChannelEnd as CoreChannelEnd, ChannelOrder as CoreChannelOrder,
    ChannelState as CoreChannelState, Packet as CorePacket, PacketCommitmentHash as CorePacketCommitmentHash,
    Sequence as CoreSequence,
};
use codechain_core::ibc::client_02::types::{ClientState as CoreClientState, ConsensusState as CoreConsensusState};
use codechain_core::ibc::connection_03::types::{
    ConnectionEnd as CoreConnectionEnd, ConnectionIdentifiersInClient as CoreConnectionIdentifiersInClient,
    ConnectionState as CoreConnectionState,
};
use primitives::{Bytes, H256};
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
pub struct ConnectionIdentifiersInClient(Vec<String>);

impl FromCore<CoreConnectionIdentifiersInClient> for ConnectionIdentifiersInClient {
    fn from_core(core: CoreConnectionIdentifiersInClient) -> Self {
        ConnectionIdentifiersInClient(core.into_vec())
    }
}

#[derive(Debug, Serialize)]
pub enum ChannelState {
    INIT,
    TRYOPEN,
    OPEN,
    CLOSED,
}

impl FromCore<CoreChannelState> for ChannelState {
    fn from_core(core: CoreChannelState) -> Self {
        match core {
            CoreChannelState::INIT => ChannelState::INIT,
            CoreChannelState::TRYOPEN => ChannelState::TRYOPEN,
            CoreChannelState::OPEN => ChannelState::OPEN,
            CoreChannelState::CLOSED => ChannelState::CLOSED,
        }
    }
}

#[derive(Debug, Serialize)]
pub enum ChannelOrder {
    ORDERED,
    UNORDERED,
}

impl FromCore<CoreChannelOrder> for ChannelOrder {
    fn from_core(core: CoreChannelOrder) -> Self {
        match core {
            CoreChannelOrder::ORDERED => ChannelOrder::ORDERED,
            CoreChannelOrder::UNORDERED => ChannelOrder::UNORDERED,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChannelEnd {
    pub state: ChannelState,
    pub ordering: ChannelOrder,
    pub counterparty_port_identifier: Identifier,
    pub counterparty_channel_identifier: Identifier,
    pub connection_hops: Vec<Identifier>,
    pub version: Identifier,
}

impl FromCore<CoreChannelEnd> for ChannelEnd {
    fn from_core(core: CoreChannelEnd) -> Self {
        ChannelEnd {
            state: ChannelState::from_core(core.state),
            ordering: ChannelOrder::from_core(core.ordering),
            counterparty_port_identifier: core.counterparty_port_identifier,
            counterparty_channel_identifier: core.counterparty_channel_identifier,
            connection_hops: core.connection_hops,
            version: core.version,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Packet {
    pub sequence: u64,
    pub timeout_height: u64,
    pub source_port: Identifier,
    pub source_channel: Identifier,
    pub dest_port: Identifier,
    pub dest_channel: Identifier,
    pub data: Bytes,
}

// Packet doesn't have to be FromCore
impl Packet {
    pub fn from_core(core: CorePacket) -> Self {
        Packet {
            sequence: core.sequence.raw,
            timeout_height: core.timeout_height,
            source_port: core.source_port,
            source_channel: core.source_channel,
            dest_port: core.dest_port,
            dest_channel: core.dest_channel,
            data: core.data,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sequence {
    pub raw: u64,
}

impl FromCore<CoreSequence> for Sequence {
    fn from_core(core: CoreSequence) -> Self {
        Sequence {
            raw: core.raw,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PacketCommitmentHash {
    pub raw: H256,
}

impl FromCore<CorePacketCommitmentHash> for PacketCommitmentHash {
    fn from_core(core: CorePacketCommitmentHash) -> Self {
        PacketCommitmentHash {
            raw: core.raw,
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AcknowledgementHash {
    pub raw: H256,
}

impl FromCore<CoreAcknowledgementHash> for AcknowledgementHash {
    fn from_core(core: CoreAcknowledgementHash) -> Self {
        AcknowledgementHash {
            raw: core.raw,
        }
    }
}
