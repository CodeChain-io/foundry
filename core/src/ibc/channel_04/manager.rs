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

use super::types::{ChannelEnd, ChannelOrder, ChannelState, Sequence};
use super::{channel_capability_path, channel_path, next_sequence_recv_path, next_sequence_send_path, DEFAULT_PORT};
use crate::ibc;
use crate::ibc::connection_03::path as connection_path;
use crate::ibc::connection_03::types::{ConnectionEnd, ConnectionState};
use crate::ibc::Identifier;

pub struct Manager<'a> {
    ctx: &'a mut dyn ibc::Context,
}

/// Temporary dummy functions for port05
fn port05_generate() -> Identifier {
    "".to_owned()
}

#[allow(unused_variables, dead_code)]
fn port05_authenticate(key: Identifier) -> bool {
    true
}

/// For all functions, there are some difference from the spec.
/// 1. They take only single Identifier as connection, since we won't consider the `hop`.
/// 2. They take no ports : All ports will be considered as DEFAULT_PORT.
impl<'a> Manager<'a> {
    pub fn new(ctx: &'a mut dyn ibc::Context) -> Self {
        Manager {
            ctx,
        }
    }

    pub fn chan_open_init(
        &mut self,
        order: ChannelOrder,
        connection: Identifier,
        channel_identifier: Identifier,
        counterparty_channel_identifier: Identifier,
        version: String,
    ) -> Result<Identifier, String> {
        let kv_store = self.ctx.get_kv_store_mut();

        // It is ok to be in any state, since here we do 'optimistic' handshake, where we establish a channel while the connection is not established completely.
        // Thus we check only the existence.
        let _: ConnectionEnd = rlp::decode(
            &kv_store.get(&connection_path(&connection)).ok_or_else(|| "Connection doesn't exist".to_owned())?,
        )
        .expect("Illformed connection end stored in the DB");

        let channel = ChannelEnd {
            state: ChannelState::INIT,
            ordering: order,
            counterparty_port_identifier: DEFAULT_PORT.to_string(),
            counterparty_channel_identifier,
            connection_hops: vec![connection],
            version,
        };

        if kv_store.insert(&channel_path(DEFAULT_PORT, &channel_identifier), &rlp::encode(&channel)).is_some() {
            return Err("Channel exists".to_owned())
        }

        let key = port05_generate();
        assert!(kv_store
            .insert(&channel_capability_path(DEFAULT_PORT, &channel_identifier), &rlp::encode(&key))
            .is_none());

        assert!(kv_store
            .insert(
                &next_sequence_send_path(DEFAULT_PORT, &channel_identifier),
                &rlp::encode(&Sequence {
                    raw: 1
                })
            )
            .is_none());

        assert!(kv_store
            .insert(
                &next_sequence_recv_path(DEFAULT_PORT, &channel_identifier),
                &rlp::encode(&Sequence {
                    raw: 1
                })
            )
            .is_none());

        Ok(key)
    }
}
