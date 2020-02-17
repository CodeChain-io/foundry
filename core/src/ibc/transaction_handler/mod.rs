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
use ckey::{Address, Public};
use cstate::{StateResult, TopLevelState};
use ctypes::errors::RuntimeError;
use ibc::client_02 as ibc_client;
use ibc::client_02::foundry as ibc_foundry;
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
        } => create_client(&mut context, &id, kind, &consensus_state),
        Datagram::UpdateClient {
            id,
            header,
        } => update_client(&mut context, &id, &header),
        Datagram::ConnOpenInit {
            identifier,
            desired_counterparty_connection_identifier,
            counterparty_prefix,
            client_identifier,
            counterparty_client_identifier,
        } => {
            let connection_manager = ibc_connection::Manager::new();
            connection_manager
                .handle_open_init(
                    &mut context,
                    identifier,
                    desired_counterparty_connection_identifier,
                    counterparty_prefix,
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
            let mut context = ibc_context::TopLevelContext::new(state, current_block_number);
            let connection_manager = ibc_connection::Manager::new();
            connection_manager
                .handle_open_try(
                    &mut context,
                    desired_identifier,
                    counterparty_connection_identifier,
                    counterparty_prefix,
                    counterparty_client_identifier,
                    client_identifier,
                    proof_init,
                    proof_consensus,
                    proof_height,
                    consensus_height,
                )
                .map_err(|err| RuntimeError::IBC(format!("ConnOpenTry: {}", err)).into())
        }
    }
}


fn create_client(
    ctx: &mut dyn ibc::Context,
    id: &str,
    kind: ibc_client::Kind,
    consensus_state: &[u8],
) -> StateResult<()> {
    let client_manager = ibc_client::Manager::new();
    if kind != ibc_client::KIND_FOUNDRY {
        return Err(RuntimeError::IBC(format!("CreateClient has invalid type {}", kind)).into())
    }
    let rlp = rlp::Rlp::new(consensus_state);
    let foundry_consensus_state: ibc_foundry::ConsensusState = match rlp.as_val() {
        Ok(cs) => cs,
        Err(err) => {
            return Err(RuntimeError::IBC(format!("CreateClient failed to decode consensus state {}", err)).into())
        }
    };

    client_manager
        .create(ctx, id, &foundry_consensus_state)
        .map_err(|err| RuntimeError::IBC(format!("CreateClient: {:?}", err)))?;
    Ok(())
}

fn update_client(ctx: &mut dyn ibc::Context, id: &str, header: &[u8]) -> StateResult<()> {
    let client_manager = ibc_client::Manager::new();
    let client_state = client_manager.query(ctx, id).map_err(RuntimeError::IBC)?;

    client_state.update(ctx, header).map_err(RuntimeError::IBC)?;

    Ok(())
}
