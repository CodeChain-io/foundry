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
use super::types::{CommitmentPrefix, Identifier};
use crate::ibc;
use crate::ibc::connection_03::client_connections_path;
use crate::ibc::connection_03::types::{ConnectionEnd, ConnectionIdentifiersInClient, ConnectionState};
use rlp::{Encodable, Rlp};

#[derive(Default)]
pub struct Manager {}

impl Manager {
    pub fn new() -> Self {
        Manager {}
    }

    pub fn handle_open_init(
        &self,
        ctx: &mut dyn ibc::Context,
        identifier: Identifier,
        desired_counterparty_connection_identifier: Identifier,
        counterparty_prefix: CommitmentPrefix,
        client_identifier: Identifier,
        counterparty_client_identifier: Identifier,
    ) -> Result<(), String> {
        let kv_store = ctx.get_kv_store();
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
        self.add_connection_to_client(ctx, client_identifier, identifier)?;
        Ok(())
    }

    fn add_connection_to_client(
        &self,
        ctx: &mut dyn ibc::Context,
        client_identifier: Identifier,
        connection_identifier: Identifier,
    ) -> Result<(), String> {
        let kv_store = ctx.get_kv_store();
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
