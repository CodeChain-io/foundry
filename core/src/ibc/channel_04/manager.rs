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

use super::log::{remove_packet, set_packet};
use super::types::{
    Acknowledgement, ChannelEnd, ChannelOrder, ChannelState, Packet, PacketCommitment, PacketCommitmentHash, Sequence,
};
use super::{
    channel_capability_path, channel_path, next_sequence_recv_path, next_sequence_send_path,
    packet_acknowledgement_path, packet_commitment_path, DEFAULT_PORT,
};
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
    fn query_connection(&self, connection_id: IdentifierSlice) -> Result<ConnectionEnd, String> {
        let kv_store = self.ctx.get_kv_store();
        let connection_end: ConnectionEnd = rlp::decode(
            &kv_store.get(&connection_path(&connection_id)).ok_or_else(|| "Connection doesn't exist".to_owned())?,
        )
        .expect("Illformed ConnectionEnd stored in the DB");
        Ok(connection_end)
    }

    fn check_connection_opened(&self, id: IdentifierSlice) -> Result<Identifier, String> {
        let connection_end = self.query_connection(id)?;
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
        let connection_end = self.query_connection(&connection)?;

        let expected = ChannelEnd {
            state: ChannelState::INIT,
            ordering: order,
            counterparty_port_identifier: DEFAULT_PORT.to_string(),
            counterparty_channel_identifier: channel_identifier.clone(),
            // Note: the array should be reversed in the future where `connection` becomes an array.
            connection_hops: vec![connection_end.counterparty_connection_identifier],
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
                let connection_end = self.query_connection(&previous.connection_hops[0])?;
                vec![connection_end.counterparty_connection_identifier]
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
                let connection_end = self.query_connection(&previous.connection_hops[0])?;
                vec![connection_end.counterparty_connection_identifier]
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

    pub fn chan_close_init(&mut self, channel_identifier: Identifier) -> Result<(), String> {
        let previous = self.get_previous_channel_end(DEFAULT_PORT, &channel_identifier)?;
        if previous.state == ChannelState::CLOSED {
            return Err("Channel already closed.".to_owned())
        }

        self.check_capability_key(DEFAULT_PORT, &channel_identifier)?;
        self.check_connection_opened(&previous.connection_hops[0])?;

        // Update
        let mut channel = previous;
        channel.state = ChannelState::CLOSED;
        let kv_store = self.ctx.get_kv_store_mut();
        kv_store.insert(&channel_path(DEFAULT_PORT, &channel_identifier), &rlp::encode(&channel));
        Ok(())
    }

    pub fn chan_close_confirm(
        &mut self,
        channel_identifier: Identifier,
        proof_init: Bytes,
        proof_height: u64,
    ) -> Result<(), String> {
        let previous = self.get_previous_channel_end(DEFAULT_PORT, &channel_identifier)?;
        if previous.state == ChannelState::CLOSED {
            return Err("Channel already closed.".to_owned())
        }

        self.check_capability_key(DEFAULT_PORT, &channel_identifier)?;
        let client_identifier = self.check_connection_opened(&previous.connection_hops[0])?;

        // Verification
        let expected = ChannelEnd {
            state: ChannelState::CLOSED,
            ordering: previous.ordering,
            counterparty_port_identifier: DEFAULT_PORT.to_string(),
            counterparty_channel_identifier: channel_identifier.clone(),
            connection_hops: {
                let connection_end = self.query_connection(&previous.connection_hops[0])?;
                vec![connection_end.counterparty_connection_identifier]
            },
            version: previous.version.clone(),
        };

        let client_manager = ClientManager::new(self.ctx);
        client_manager.verify_channel_state(
            &client_identifier,
            proof_height,
            proof_init,
            &previous.counterparty_port_identifier,
            &previous.counterparty_channel_identifier,
            &expected,
        )?;

        // Update
        let mut channel = previous;
        channel.state = ChannelState::CLOSED;
        self.ctx.get_kv_store_mut().insert(&channel_path(DEFAULT_PORT, &channel_identifier), &rlp::encode(&channel));
        Ok(())
    }

    // ICS Packet
    pub fn send_packet(&mut self, packet: Packet) -> Result<(), String> {
        let channel = self.get_previous_channel_end(&packet.source_port, &packet.source_channel)?;
        if channel.state == ChannelState::CLOSED {
            return Err("Channel closed.".to_owned())
        }
        if packet.dest_port != channel.counterparty_port_identifier {
            return Err("Packet's dest_port doesn't match with counterparty's channel.".to_owned())
        }
        if packet.dest_channel != channel.counterparty_channel_identifier {
            return Err("Packet's dest_channel doesn't match with counterparty's channel.".to_owned())
        }

        self.check_capability_key(&packet.source_port, &packet.source_channel)?;
        let client_identifier = self.check_connection_opened(&channel.connection_hops[0])?;

        let client_manager = ClientManager::new(self.ctx);
        let client_state = client_manager.query(&client_identifier)?;

        if packet.timeout_height <= client_state.raw.number {
            return Err("Packet carries invalid timeout_height".to_owned())
        }

        let mut next_sequence_send = self.get_sequence_send(&packet.source_port, &packet.source_channel)?;

        if packet.sequence != next_sequence_send {
            return Err(format!(
                "Packet carries invalid sequence. expected: {:?}, actual: {:?}",
                next_sequence_send, packet.sequence
            ))
        }

        next_sequence_send.raw += 1;
        let kv_store = self.ctx.get_kv_store_mut();

        kv_store.insert(
            &next_sequence_send_path(&packet.source_port, &packet.source_channel),
            &rlp::encode(&next_sequence_send),
        );
        kv_store.insert(
            &packet_commitment_path(&packet.source_port, &packet.source_channel, &packet.sequence),
            &rlp::encode(
                &PacketCommitment {
                    data: packet.data.clone(),
                    timeout: packet.timeout_height,
                }
                .hash(),
            ),
        );

        // check log.rs to understand this statement
        set_packet(self.ctx, &packet, "send");

        Ok(())
    }

    pub fn recv_packet(
        &mut self,
        packet: Packet,
        proof: Bytes,
        proof_height: u64,
        ack: Acknowledgement,
    ) -> Result<Packet, String> {
        let channel = self.get_previous_channel_end(&packet.dest_port, &packet.dest_channel)?;
        if channel.state != ChannelState::OPEN {
            return Err("Channel not opened.".to_owned())
        }
        if packet.source_port != channel.counterparty_port_identifier {
            return Err("Packet's source_port doesn't match with counterparty's channel.".to_owned())
        }
        if packet.source_channel != channel.counterparty_channel_identifier {
            return Err("Packet's source_channel doesn't match with counterparty's channel.".to_owned())
        }

        self.check_capability_key(&packet.dest_port, &packet.dest_channel)?;
        let client_identifier = self.check_connection_opened(&channel.connection_hops[0])?;
        let client_manager = ClientManager::new(self.ctx);

        let expected = PacketCommitment {
            data: packet.data.clone(),
            timeout: packet.timeout_height,
        };

        client_manager.verify_packet_data(
            &client_identifier,
            proof_height,
            proof,
            &packet.source_port,
            &packet.source_channel,
            &packet.sequence,
            &expected,
        )?;

        if self.ctx.get_current_height() >= packet.timeout_height {
            return Err("Packet timeout.".to_owned())
        }

        let mut next_sequence_recv = self.get_sequence_recv(&packet.dest_port, &packet.dest_channel)?;
        let kv_store = self.ctx.get_kv_store_mut();
        if channel.ordering == ChannelOrder::ORDERED {
            kv_store.insert(
                &packet_acknowledgement_path(&packet.dest_port, &packet.dest_channel, &packet.sequence),
                &rlp::encode(&ack.hash()),
            );

            if packet.sequence != next_sequence_recv {
                return Err(format!(
                    "Packet carries invalid sequence. expected: {:?} actal: {:?}",
                    next_sequence_recv, packet.sequence
                ))
            }
            next_sequence_recv.raw += 1;
            kv_store.insert(
                &next_sequence_recv_path(&packet.dest_port, &packet.dest_channel),
                &rlp::encode(&next_sequence_recv),
            );
        } else {
            panic!("PoC doesn't support UNORDERED channel");
        }

        // check log.rs to understand this statement
        // Unlike send, we just overwrite an old event.
        remove_packet(self.ctx, &packet.dest_port, &packet.dest_channel, "recv");
        set_packet(self.ctx, &packet, "recv");

        Ok(packet)
    }

    pub fn acknowledge_packet(
        &mut self,
        packet: Packet,
        ack: Acknowledgement,
        proof: Bytes,
        proof_height: u64,
    ) -> Result<Packet, String> {
        let channel = self.get_previous_channel_end(&packet.source_port, &packet.source_channel)?;
        if channel.state != ChannelState::OPEN {
            return Err("Channel not opened.".to_owned())
        }
        if packet.dest_port != channel.counterparty_port_identifier {
            return Err("Packet's dest_port doesn't match with counterparty's channel.".to_owned())
        }
        if packet.dest_channel != channel.counterparty_channel_identifier {
            return Err("Packet's dest_channel doesn't match with counterparty's channel.".to_owned())
        }

        self.check_capability_key(&packet.source_port, &packet.source_channel)?;

        let kv_store = self.ctx.get_kv_store();
        let packet_commitment: PacketCommitmentHash = rlp::decode(
            &kv_store
                .get(&packet_commitment_path(&packet.source_port, &packet.source_channel, &packet.sequence))
                .ok_or_else(|| "Packet commitment not found".to_owned())?,
        )
        .expect("Illformed Packet commitment stored in the DB");

        let expected_to_store = PacketCommitment {
            data: packet.data.clone(),
            timeout: packet.timeout_height,
        }
        .hash();
        if packet_commitment != expected_to_store {
            return Err("This acknowledge is not an expected one.".to_owned())
        }

        let client_identifier = self.check_connection_opened(&channel.connection_hops[0])?;
        let client_manager = ClientManager::new(self.ctx);

        client_manager.verify_packet_acknowledgment(
            &client_identifier,
            proof_height,
            proof,
            &packet.dest_port,
            &packet.dest_channel,
            &packet.sequence,
            &ack,
        )?;

        let kv_store = self.ctx.get_kv_store_mut();
        kv_store.remove(&packet_commitment_path(&packet.source_port, &packet.source_channel, &packet.sequence));

        // check log.rs to understand this statement
        remove_packet(self.ctx, &packet.source_port, &packet.source_channel, "send");

        Ok(packet)
    }
}