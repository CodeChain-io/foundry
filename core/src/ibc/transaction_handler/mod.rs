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
use ibc::client_02 as ibc_client;
use ibc::connection_03 as ibc_connection;
use ibc::context as ibc_context;
use rlp::{Decodable, Rlp};

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
            let mut client_manager = ibc_client::Manager::new(&mut context);
            client_manager.update(&id, header).map_err(|err| RuntimeError::IBC(format!("CreateClient: {:?}", err)))?;
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
    }
}
