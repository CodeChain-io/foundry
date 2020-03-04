// Copyright 2020 Kodebox, Inc.
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

mod datagrams;

use self::datagrams::Datagram;
use crate::ibc;
use crate::ibc::commitment_23::types::CommitmentPrefix;
use ckey::{Address, Public};
use cstate::{StateResult, TopLevelState};
use ctypes::errors::RuntimeError;
use ibc::channel_04 as ibc_channel;
use ibc::client_02 as ibc_client;
use ibc::connection_03 as ibc_connection;
use ibc::context as ibc_context;
use rlp::{Decodable, Rlp};
use rustc_hex::ToHex;

pub fn execute(
    bytes: &[u8],
    state: &mut TopLevelState,
    _fee_payer: &Address,
    _sender_public: &Public,
    current_block_number: u64,
) -> StateResult<()> {
    let mut context = ibc_context::TopLevelContext::new(state, current_block_number);
    let datagram = Datagram::decode(&Rlp::new(bytes)).expect("Verification passed");
    match datagram {
        Datagram::CreateClient {
            id,
            kind,
            consensus_state,
            data,
        } => {
            let mut client_manager = ibc_client::Manager::new(&mut context);
            // We support Foundry light client only
            if kind != ibc_client::KIND_FOUNDRY {
                return Err(RuntimeError::IBC(format!("CreateClient has invalid type {}", kind)).into())
            }
            client_manager
                .create(&id, consensus_state, data)
                .map_err(|err| RuntimeError::IBC(format!("CreateClient: {:?}", err)))?;
            Ok(())
        }
        Datagram::UpdateClient {
            id,
            header,
        } => {
            cdebug!(IBC, "Update client {} {}", id, header.to_hex());
            let mut client_manager = ibc_client::Manager::new(&mut context);
            client_manager.update(&id, header).map_err(|err| RuntimeError::IBC(format!("UpdateClient: {:?}", err)))?;
            Ok(())
        }
        Datagram::ConnOpenInit {
            identifier,
            desired_counterparty_connection_identifier,
            counterparty_prefix,
            client_identifier,
            counterparty_client_identifier,
        } => {
            let mut connection_manager = ibc_connection::Manager::new(&mut context);
            connection_manager
                .handle_open_init(
                    identifier,
                    desired_counterparty_connection_identifier,
                    CommitmentPrefix {
                        raw: counterparty_prefix,
                    },
                    client_identifier,
                    counterparty_client_identifier,
                )
                .map_err(|err| RuntimeError::IBC(format!("ConnOpenInit: {}", err)).into())
        }
        Datagram::ConnOpenTry {
            desired_identifier,
            counterparty_connection_identifier,
            counterparty_prefix,
            counterparty_client_identifier,
            client_identifier,
            proof_init,
            proof_consensus,
            proof_height,
            consensus_height,
        } => {
            let mut connection_manager = ibc_connection::Manager::new(&mut context);

            connection_manager
                .handle_open_try(
                    desired_identifier,
                    counterparty_connection_identifier,
                    CommitmentPrefix {
                        raw: counterparty_prefix,
                    },
                    counterparty_client_identifier,
                    client_identifier,
                    proof_init,
                    proof_consensus,
                    proof_height,
                    consensus_height,
                )
                .map_err(|err| RuntimeError::IBC(format!("ConnOpenTry: {}", err)).into())
        }
        Datagram::ConnOpenAck {
            identifier,
            proof_try,
            proof_consensus,
            proof_height,
            consensus_height,
        } => {
            let mut connection_manager = ibc_connection::Manager::new(&mut context);
            connection_manager
                .handle_open_ack(identifier, proof_try, proof_consensus, proof_height, consensus_height)
                .map_err(|err| RuntimeError::IBC(format!("ConnOpenAck: {}", err)).into())
        }
        Datagram::ConnOpenConfirm {
            identifier,
            proof_ack,
            proof_height,
        } => {
            let mut connection_manager = ibc_connection::Manager::new(&mut context);
            connection_manager
                .handle_open_confirm(identifier, proof_ack, proof_height)
                .map_err(|err| RuntimeError::IBC(format!("ConnOpenConfirm: {}", err)).into())
        }
        Datagram::ChanOpenInit {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager
                .chan_open_init(
                    {
                        if raw.order == 1 {
                            ibc::channel_04::types::ChannelOrder::ORDERED
                        } else {
                            ibc::channel_04::types::ChannelOrder::UNORDERED
                        }
                    },
                    raw.connection,
                    raw.channel_identifier,
                    raw.counterparty_channel_identifier,
                    raw.version,
                )
                .map_err(|err| RuntimeError::IBC(format!("ChanOpenInit: {}", err)))?;
            Ok(())
        }
        Datagram::ChanOpenTry {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager
                .chan_open_try(
                    {
                        if raw.order == 1 {
                            ibc::channel_04::types::ChannelOrder::ORDERED
                        } else {
                            ibc::channel_04::types::ChannelOrder::UNORDERED
                        }
                    },
                    raw.connection,
                    raw.channel_identifier,
                    raw.counterparty_channel_identifier,
                    raw.version,
                    raw.counterparty_version,
                    raw.proof_init,
                    raw.proof_height,
                )
                .map_err(|err| RuntimeError::IBC(format!("ChanOpenTry: {}", err)))?;
            Ok(())
        }
        Datagram::ChanOpenAck {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager
                .chan_open_ack(raw.channel_identifier, raw.counterparty_version, raw.proof_try, raw.proof_height)
                .map_err(|err| RuntimeError::IBC(format!("ChanOpenAck: {}", err)))?;
            Ok(())
        }
        Datagram::ChanOpenConfirm {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager
                .chan_open_confirm(raw.channel_identifier, raw.proof_ack, raw.proof_height)
                .map_err(|err| RuntimeError::IBC(format!("ChanOpenConfirm: {}", err)))?;
            Ok(())
        }
        Datagram::ChanCloseInit {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager
                .chan_close_init(raw.channel_identifier)
                .map_err(|err| RuntimeError::IBC(format!("ChanCloseInit: {}", err)))?;
            Ok(())
        }
        Datagram::ChanCloseConfirm {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager
                .chan_close_confirm(raw.channel_identifier, raw.proof_init, raw.proof_height)
                .map_err(|err| RuntimeError::IBC(format!("ChanCloseConfirm: {}", err)))?;
            Ok(())
        }
        Datagram::SendPacket {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager.send_packet(raw.packet).map_err(|err| RuntimeError::IBC(format!("SendPacket: {}", err)))?;
            Ok(())
        }
        Datagram::RecvPacket {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager
                .recv_packet(raw.packet, raw.proof, raw.proof_height, ibc::channel_04::types::Acknowledgement {
                    raw: raw.ack,
                })
                .map_err(|err| RuntimeError::IBC(format!("RecvPacket: {}", err)))?;
            Ok(())
        }
        Datagram::AcknowledgePacket {
            raw,
        } => {
            let mut channel_manager = ibc_channel::Manager::new(&mut context);
            channel_manager
                .acknowledge_packet(
                    raw.packet,
                    ibc::channel_04::types::Acknowledgement {
                        raw: raw.ack,
                    },
                    raw.proof,
                    raw.proof_height,
                )
                .map_err(|err| RuntimeError::IBC(format!("AcknowledgePacket: {}", err)))?;
            Ok(())
        }
    }
}
