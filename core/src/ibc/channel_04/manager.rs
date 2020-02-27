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
use crate::ibc::{Identifier, IdentifierSlice};
use ibc::client_02::Manager as ClientManager;
use primitives::Bytes;

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

    // Utility functions
    fn check_connection_opened(&self, id: IdentifierSlice) -> Result<Identifier, String> {
        let kv_store = self.ctx.get_kv_store();
        let connection_end: ConnectionEnd =
            rlp::decode(&kv_store.get(&connection_path(&id)).ok_or_else(|| "Connection doesn't exist".to_owned())?)
                .expect("Illformed ConnectionEnd stored in the DB");

        if connection_end.state != ConnectionState::OPEN {
            return Err("Connection not opened".to_owned())
        }
        Ok(connection_end.client_identifier)
    }

    fn check_capability_key(&self, port: IdentifierSlice, channel: IdentifierSlice) -> Result<(), String> {
        let kv_store = self.ctx.get_kv_store();
        let key: String = rlp::decode(
            &kv_store
                .get(&channel_capability_path(port, channel))
                .ok_or_else(|| "capability key not found".to_owned())?,
        )
        .expect("Illformed capability key stored in the DB");

        if !port05_authenticate(key) {
            return Err("Invalid capability key".to_owned())
        }
        Ok(())
    }

    fn get_previous_channel_end(&self, port: IdentifierSlice, channel: IdentifierSlice) -> Result<ChannelEnd, String> {
        let kv_store = self.ctx.get_kv_store();
        let previous: ChannelEnd =
            rlp::decode(&kv_store.get(&channel_path(port, channel)).ok_or_else(|| "ChannelEnd not found".to_owned())?)
                .expect("Illformed ChannelEnd stored in the DB");

        Ok(previous)
    }

    fn get_sequence_send(&self, port: IdentifierSlice, channel: IdentifierSlice) -> Result<Sequence, String> {
        let kv_store = self.ctx.get_kv_store();
        let sequence: Sequence = rlp::decode(
            &kv_store
                .get(&next_sequence_send_path(port, channel))
                .ok_or_else(|| "Next sequence(Send) not found".to_owned())?,
        )
        .expect("Illformed Sequence stored in the DB");
        Ok(sequence)
    }

    fn get_sequence_recv(&self, port: IdentifierSlice, channel: IdentifierSlice) -> Result<Sequence, String> {
        let kv_store = self.ctx.get_kv_store();
        let sequence: Sequence = rlp::decode(
            &kv_store
                .get(&next_sequence_recv_path(port, channel))
                .ok_or_else(|| "Next sequence(Recv) not found".to_owned())?,
        )
        .expect("Illformed Sequence stored in the DB");
        Ok(sequence)
    }

    // ICS Channel
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

    pub fn chan_open_try(
        &mut self,
        order: ChannelOrder,
        connection: Identifier,
        channel_identifier: Identifier,
        counterparty_channel_identifier: Identifier,
        version: String,
        counterparty_version: String,
        proof_init: Bytes,
        proof_height: u64,
    ) -> Result<Identifier, String> {
        let kv_store = self.ctx.get_kv_store();

        let previous = kv_store.get(&channel_path(&channel_identifier, DEFAULT_PORT));

        // A trivial case is establishing a channel from scratch, so there won't be an existing one while responding counterparty's 'init' with 'try'.
        // However, if it exists, we just check that our last trial ('init') was intended to make a same channel as the counterparty chain's 'init' did.
        // And then overwrites it.
        if let Some(x) = previous {
            let channel_end: ChannelEnd = rlp::decode(&x).expect("Illformed connection end stored in the DB");
            let expected = ChannelEnd {
                state: ChannelState::INIT,
                ordering: order,
                counterparty_port_identifier: DEFAULT_PORT.to_string(),
                counterparty_channel_identifier: counterparty_channel_identifier.clone(),
                connection_hops: vec![connection.clone()],
                version: version.clone(),
            };
            if channel_end != expected {
                return Err("There already exists ChannelEnd on which open_try() can't be conducted.".to_owned())
            }
        }

        let client_identifier = self.check_connection_opened(&connection)?;

        let expected = ChannelEnd {
            state: ChannelState::INIT,
            ordering: order,
            counterparty_port_identifier: DEFAULT_PORT.to_string(),
            counterparty_channel_identifier: channel_identifier.clone(),
            // Note: the array should be reversed in the future where `connection` becomes an array.
            connection_hops: vec![connection.clone()],
            version: counterparty_version,
        };

        let client_manager = ClientManager::new(self.ctx);
        client_manager.verify_channel_state(
            &client_identifier,
            proof_height,
            proof_init,
            DEFAULT_PORT,
            &counterparty_channel_identifier,
            &expected,
        )?;

        let channel = ChannelEnd {
            state: ChannelState::TRYOPEN,
            ordering: order,
            counterparty_port_identifier: DEFAULT_PORT.to_string(),
            counterparty_channel_identifier,
            connection_hops: vec![connection],
            version,
        };

        let kv_store = self.ctx.get_kv_store_mut();

        kv_store.insert(&channel_path(DEFAULT_PORT, &channel_identifier), &rlp::encode(&channel));

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

    pub fn chan_open_ack(
        &mut self,
        channel_identifier: Identifier,
        counterparty_version: String,
        proof_try: Bytes,
        proof_height: u64,
    ) -> Result<(), String> {
        let previous = self.get_previous_channel_end(DEFAULT_PORT, &channel_identifier)?;
        if previous.state != ChannelState::INIT && previous.state != ChannelState::TRYOPEN {
            return Err("Channel already established".to_owned())
        }
        self.check_capability_key(DEFAULT_PORT, &channel_identifier)?;
        let client_identifier = self.check_connection_opened(&previous.connection_hops[0])?;

        // Verification
        let expected = ChannelEnd {
            state: ChannelState::TRYOPEN,
            ordering: previous.ordering,
            counterparty_port_identifier: DEFAULT_PORT.to_string(),
            counterparty_channel_identifier: channel_identifier.clone(),
            connection_hops: {
                let mut x = previous.connection_hops.clone();
                x.reverse();
                x
            },
            version: counterparty_version.clone(),
        };

        let client_manager = ClientManager::new(self.ctx);
        client_manager.verify_channel_state(
            &client_identifier,
            proof_height,
            proof_try,
            &previous.counterparty_port_identifier,
            &previous.counterparty_channel_identifier,
            &expected,
        )?;

        // Update
        let mut channel = previous;
        channel.state = ChannelState::OPEN;
        channel.version = counterparty_version;
        self.ctx.get_kv_store_mut().insert(&channel_path(DEFAULT_PORT, &channel_identifier), &rlp::encode(&channel));
        Ok(())
    }

    pub fn chan_open_confirm(
        &mut self,
        channel_identifier: Identifier,
        proof_ack: Bytes,
        proof_height: u64,
    ) -> Result<(), String> {
        let previous = self.get_previous_channel_end(DEFAULT_PORT, &channel_identifier)?;
        if previous.state != ChannelState::TRYOPEN {
            return Err("ChannelState is on state on which open_confirm() can't be conducted.".to_owned())
        }

        self.check_capability_key(DEFAULT_PORT, &channel_identifier)?;
        let client_identifier = self.check_connection_opened(&previous.connection_hops[0])?;

        // Verification
        let expected = ChannelEnd {
            state: ChannelState::OPEN,
            ordering: previous.ordering,
            counterparty_port_identifier: DEFAULT_PORT.to_string(),
            counterparty_channel_identifier: channel_identifier.clone(),
            connection_hops: {
                let mut x = previous.connection_hops.clone();
                x.reverse();
                x
            },
            version: previous.version.clone(),
        };

        let client_manager = ClientManager::new(self.ctx);
        client_manager.verify_channel_state(
            &client_identifier,
            proof_height,
            proof_ack,
            &previous.counterparty_port_identifier,
            &previous.counterparty_channel_identifier,
            &expected,
        )?;

        // Update
        let mut channel = previous;
        channel.state = ChannelState::OPEN;
        self.ctx.get_kv_store_mut().insert(&channel_path(DEFAULT_PORT, &channel_identifier), &rlp::encode(&channel));
        Ok(())
    }
}
