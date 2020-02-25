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

use super::path as connection_path;
use crate::ibc;
use crate::ibc::commitment_23::types::{get_commiment_prefix, CommitmentPrefix};
use crate::ibc::connection_03::client_connections_path;
use crate::ibc::connection_03::types::{ConnectionEnd, ConnectionIdentifiersInClient, ConnectionState};
use crate::ibc::{Identifier, IdentifierSlice};
use ibc::client_02::Manager as ClientManager;
use primitives::Bytes;
use rlp::{Encodable, Rlp};

pub struct Manager<'a> {
    ctx: &'a mut dyn ibc::Context,
}

impl<'a> Manager<'a> {
    pub fn new(ctx: &'a mut dyn ibc::Context) -> Self {
        Manager {
            ctx,
        }
    }

    pub fn handle_open_init(
        &mut self,
        identifier: Identifier,
        desired_counterparty_connection_identifier: Identifier,
        counterparty_prefix: CommitmentPrefix,
        client_identifier: Identifier,
        counterparty_client_identifier: Identifier,
    ) -> Result<(), String> {
        let kv_store = self.ctx.get_kv_store_mut();
        if kv_store.contains_key(&connection_path(&identifier)) {
            return Err("Connection exist".to_owned())
        }
        let state = ConnectionState::INIT;
        let connection = ConnectionEnd {
            state,
            counterparty_connection_identifier: desired_counterparty_connection_identifier,
            counterparty_prefix,
            client_identifier: client_identifier.clone(),
            counterparty_client_identifier,
        };
        kv_store.insert(&connection_path(&identifier), &connection.rlp_bytes());
        self.add_connection_to_client(client_identifier, identifier)?;
        Ok(())
    }

    // We all following ICS spec.
    #[allow(clippy::too_many_arguments)]
    pub fn handle_open_try(
        &mut self,
        desired_identifier: Identifier,
        counterparty_connection_identifier: Identifier,
        counterparty_prefix: CommitmentPrefix,
        counterparty_client_identifier: Identifier,
        client_identifier: Identifier,
        proof_init: Bytes,
        proof_consensus: Bytes,
        proof_height: u64,
        consensus_height: u64,
    ) -> Result<(), String> {
        let current_height = self.ctx.get_current_height();
        if consensus_height > current_height {
            return Err(format!(
                "Consensus height {} is greater than current height {}",
                consensus_height, current_height
            ))
        }
        let expected = ConnectionEnd {
            state: ConnectionState::INIT,
            counterparty_connection_identifier: desired_identifier.clone(),
            counterparty_prefix: get_commiment_prefix(),
            client_identifier: counterparty_client_identifier.clone(),
            counterparty_client_identifier: client_identifier.clone(),
        };

        let connection = ConnectionEnd {
            state: ConnectionState::TRYOPEN,
            counterparty_connection_identifier: counterparty_client_identifier.clone(),
            counterparty_prefix: counterparty_prefix.clone(),
            client_identifier: client_identifier.clone(),
            counterparty_client_identifier: counterparty_client_identifier.clone(),
        };

        let client_manager = ClientManager::new(self.ctx);
        client_manager.verify_connection_state(
            &counterparty_client_identifier,
            proof_height,
            proof_init,
            &desired_identifier,
            &expected,
        )?;

        if let Some(previous_connection_end) = self.query(&desired_identifier) {
            let expected_init = ConnectionEnd {
                state: ConnectionState::INIT,
                counterparty_connection_identifier,
                counterparty_prefix,
                client_identifier,
                counterparty_client_identifier,
            };
            if previous_connection_end != expected_init {
                return Err(format!(
                    "Invalid previous connection status: previous: {:?}, expected: {:?}",
                    previous_connection_end, expected_init
                ))
            }
        }

        let kv_store = self.ctx.get_kv_store_mut();
        kv_store.insert(&connection_path(&desired_identifier), &connection.rlp_bytes());
        Ok(())
    }

    pub fn handle_open_ack(
        &mut self,
        identifier: Identifier,
        proof_try: Bytes,
        proof_consensus: Bytes,
        proof_height: u64,
        consensus_height: u64,
    ) -> Result<(), String> {
        let current_height = self.ctx.get_current_height();
        if consensus_height > current_height {
            return Err(format!(
                "Consensus height {} is greater than current height {}",
                consensus_height, current_height
            ))
        }
        let mut connection = self
            .query(&identifier)
            .ok_or_else(|| format!("Cannot find connection with the identifier: {}", identifier))?;

        if connection.state != ConnectionState::INIT && connection.state != ConnectionState::TRYOPEN {
            return Err(format!("Invalid connection state expected INIT or TRYOPEN but found {:?}", connection.state))
        }
        let expected = ConnectionEnd {
            state: ConnectionState::TRYOPEN,
            counterparty_connection_identifier: identifier.clone(),
            counterparty_prefix: get_commiment_prefix(),
            client_identifier: connection.counterparty_client_identifier.clone(),
            counterparty_client_identifier: connection.client_identifier.clone(),
        };

        let client_manager = ClientManager::new(self.ctx);
        client_manager.verify_connection_state(
            &connection.client_identifier,
            proof_height,
            proof_try,
            &identifier,
            &expected,
        )?;

        connection.state = ConnectionState::OPEN;
        let kv_store = self.ctx.get_kv_store_mut();
        let path = connection_path(&identifier);
        kv_store.insert(&path, &connection.rlp_bytes());

        Ok(())
    }

    pub fn handle_open_confirm(
        &mut self,
        identifier: Identifier,
        proof_ack: Bytes,
        proof_height: u64,
    ) -> Result<(), String> {
        let mut connection = self
            .query(&identifier)
            .ok_or_else(|| format!("Cannot find connection with the identifier: {}", identifier))?;
        if connection.state != ConnectionState::TRYOPEN {
            return Err(format!("Invalid connection state expected TRYOPEN but found {:?}", connection.state))
        }

        let expected = ConnectionEnd {
            state: ConnectionState::OPEN,
            counterparty_connection_identifier: identifier.clone(),
            counterparty_prefix: get_commiment_prefix(),
            client_identifier: connection.counterparty_client_identifier.clone(),
            counterparty_client_identifier: connection.client_identifier.clone(),
        };

        let client_manager = ClientManager::new(self.ctx);
        client_manager.verify_connection_state(
            &connection.client_identifier,
            proof_height,
            proof_ack,
            &identifier,
            &expected,
        )?;

        connection.state = ConnectionState::OPEN;
        let kv_store = self.ctx.get_kv_store_mut();
        let path = connection_path(&identifier);
        kv_store.insert(&path, &connection.rlp_bytes());

        Ok(())
    }

    fn query(&mut self, identifier: IdentifierSlice) -> Option<ConnectionEnd> {
        let kv_store = self.ctx.get_kv_store();

        let path = connection_path(&identifier);
        let raw = kv_store.get(&path)?;
        let connection_end = rlp::decode(&raw).expect("Only the connection code can save the code");
        Some(connection_end)
    }

    fn add_connection_to_client(
        &mut self,
        client_identifier: Identifier,
        connection_identifier: Identifier,
    ) -> Result<(), String> {
        let kv_store = self.ctx.get_kv_store_mut();
        let bytes =
            kv_store.get(&client_connections_path(&client_identifier)).ok_or_else(|| "Connection exist".to_owned())?;
        let rlp = Rlp::new(&bytes);
        let mut conns: ConnectionIdentifiersInClient = rlp.as_val().expect("data from DB");

        conns.add(connection_identifier);

        kv_store.insert(&client_connections_path(&client_identifier), &rlp::encode(&conns));
        Ok(())
    }
}
