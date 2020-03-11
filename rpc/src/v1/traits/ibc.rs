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

use super::super::types::{
    AcknowledgementHash, ChannelEnd, ClientState, ConnectionEnd, ConnectionIdentifiersInClient, ConsensusState,
    IBCQuery, Packet, PacketCommitmentHash, Sequence,
};
use jsonrpc_core::Result;
use primitives::Bytes;

#[rpc(server)]
pub trait IBC {
    /// Gets client state
    #[rpc(name = "ibc_query_client_state")]
    fn query_client_state(&self, client_id: String, block_number: Option<u64>)
        -> Result<Option<IBCQuery<ClientState>>>;

    /// Gets consensus state on arbitrary number
    #[rpc(name = "ibc_query_consensus_state")]
    fn query_consensus_state(
        &self,
        client_id: String,
        counterparty_block_number: u64,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<ConsensusState>>>;

    /// Compose an ICS header that updates Foundry light client (not me, but being in some counterparty chain)
    /// from block_number-1 to block_number. It will stay opaque until it gets finally delieverd to Foundry light client.
    #[rpc(name = "ibc_compose_header")]
    fn compose_header(&self, block_number: u64) -> Result<Option<Bytes>>;

    #[rpc(name = "ibc_query_connection")]
    fn query_connection(
        &self,
        identifier: String,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<ConnectionEnd>>>;

    #[rpc(name = "ibc_query_client_connections")]
    fn query_client_connections(
        &self,
        client_identifier: String,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<ConnectionIdentifiersInClient>>>;

    #[rpc(name = "ibc_query_channel_end")]
    fn query_channel_end(
        &self,
        port_id: String,
        channel_id: String,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<ChannelEnd>>>;

    #[rpc(name = "ibc_query_packet_commitment")]
    fn query_packet_commitment(
        &self,
        port_id: String,
        channel_id: String,
        sequence: u64,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<PacketCommitmentHash>>>;

    #[rpc(name = "ibc_query_packet_acknowledgement")]
    fn query_packet_acknowledgement(
        &self,
        port_id: String,
        channel_id: String,
        sequence: u64,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<AcknowledgementHash>>>;

    #[rpc(name = "ibc_query_next_sequence_recv")]
    fn query_next_sequence_recv(
        &self,
        port_id: String,
        channel_id: String,
        block_number: Option<u64>,
    ) -> Result<Option<IBCQuery<Sequence>>>;

    #[rpc(name = "ibc_query_latest_send_packet")]
    fn query_latest_send_packet(
        &self,
        port_id: String,
        channel_id: String,
        block_number: Option<u64>,
    ) -> Result<Option<Packet>>;

    #[rpc(name = "ibc_query_latest_recv_packet")]
    fn query_latest_recv_packet(
        &self,
        port_id: String,
        channel_id: String,
        block_number: Option<u64>,
    ) -> Result<Option<Packet>>;
}
