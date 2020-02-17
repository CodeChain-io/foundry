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
use super::types::{CommitmentPrefix, CommitmentProof, Identifier};
use crate::ibc;
use crate::ibc::connection_03::client_connections_path;
use crate::ibc::connection_03::types::{ConnectionEnd, ConnectionIdentifiersInClient, ConnectionState};
use rlp::{Encodable, Rlp};

pub struct Manager<'a> {
    ctx: &'a mut dyn ibc::Context,
}

// FIXME: this will be changed after implementing Vector commitment
fn get_commiment_prefix() -> String {
    "".to_owned()
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
        let kv_store = self.ctx.get_kv_store();
        if kv_store.has(&connection_path(&identifier)) {
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
        kv_store.set(&connection_path(&identifier), &connection.rlp_bytes());
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
        proof_init: CommitmentProof,
        proof_consensus: CommitmentProof,
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

        self.verify_connection_state(&connection, proof_height, proof_init, desired_identifier.clone(), &expected);

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

        let kv_store = self.ctx.get_kv_store();
        kv_store.set(&connection_path(&desired_identifier), &connection.rlp_bytes());
        Ok(())
    }

    fn query(&mut self, identifier: &str) -> Option<ConnectionEnd> {
        let kv_store = self.ctx.get_kv_store();

        let path = connection_path(&identifier);
        if kv_store.has(&path) {
            let raw = kv_store.get(&path);
            let connection_end = rlp::decode(&raw).expect("Only the connection code can save the code");
            return Some(connection_end)
        }

        None
    }

    fn verify_connection_state(
        &mut self,
        connection: &ConnectionEnd,
        proof_height: u64,
        proof: CommitmentProof,
        connection_identifier: Identifier,
        connection_end: &ConnectionEnd,
    ) -> bool {
        // check values in the connection_end
        let path = format!("connections/{}", connection_identifier);
        self.client_verify_membership(proof_height, proof, path, &rlp::encode(connection_end))
    }

    fn client_verify_membership(
        &self,
        _height: u64,
        _commitment_proof: CommitmentProof,
        _path: String,
        _value: &[u8],
    ) -> bool {
        // FIXME
        true
    }

    fn add_connection_to_client(
        &mut self,
        client_identifier: Identifier,
        connection_identifier: Identifier,
    ) -> Result<(), String> {
        let kv_store = self.ctx.get_kv_store();
        if kv_store.has(&connection_path(&connection_identifier)) {
            return Err("Connection exist".to_owned())
        }
        let bytes = kv_store.get(&client_connections_path(&client_identifier));
        let rlp = Rlp::new(&bytes);
        let mut conns: ConnectionIdentifiersInClient = rlp.as_val().expect("data from DB");

        conns.add(connection_identifier);

        kv_store.set(&client_connections_path(&client_identifier), &rlp::encode(&conns));
        Ok(())
    }
}
