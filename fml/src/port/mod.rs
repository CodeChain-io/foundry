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

pub mod client;
pub mod server;

use crate::context::{single_process_support::InstanceKey, FmlConfig};
use crate::handle::{MethodId, PortDispatcher, ServiceObjectId};
use cbsb::ipc::{multiplex, IpcRecv, IpcSend};
use std::sync::Arc;

// This module implements two important communication models: Client and Server
//
// A calls B
// => A invokes B's method with Client.
// => B handles that call in Server dispatching the packet, and returns response.
// => A receives that response.
//
// Here servcie handler simply calls the dispatcher given by the port,
// whenever it receives a new inbound call.
//
// TODO: Introduce Rust async/await to support N concurrent calls where
// N > # of handler threads

pub type SlotId = u32;
pub type PortId = u16;

const SLOT_CALL_OR_RETURN_INDICATOR: SlotId = 1000;
const DELETE_INDICATOR: MethodId = 1234;

const MULTIPLEX_INDEX_SERVER: usize = 0;
const MULTIPLEX_INDEX_CLIENT: usize = 1;

#[derive(PartialEq, Debug)]
pub struct PacketHeader {
    pub slot: SlotId,
    pub handle: ServiceObjectId,
    pub method: MethodId,
}

impl PacketHeader {
    pub fn new(buffer: &[u8]) -> Self {
        unsafe { std::ptr::read(buffer.as_ptr().cast()) }
    }

    pub fn write(&self, buffer: &mut [u8]) {
        unsafe {
            std::ptr::copy_nonoverlapping(self, buffer.as_mut_ptr().cast(), 1);
        }
    }
}

#[test]
fn encoding_packet_header() {
    let ph1 = PacketHeader {
        slot: 0x1234,
        handle: ServiceObjectId {
            trait_id: 0x9999,
            index: 0x8888,
        },
        method: 0x5678,
    };
    let mut buffer = vec![0 as u8; std::mem::size_of::<PacketHeader>()];
    ph1.write(&mut buffer);
    let ph2 = PacketHeader::new(&buffer);
    assert_eq!(ph2, ph1);
}

pub struct ServerOrClientForwarder;

impl multiplex::Forward for ServerOrClientForwarder {
    fn forward(data: &[u8]) -> usize {
        let header = PacketHeader::new(&data);
        if header.slot >= SLOT_CALL_OR_RETURN_INDICATOR {
            MULTIPLEX_INDEX_SERVER
        } else {
            MULTIPLEX_INDEX_CLIENT
        }
    }
}

pub struct Port {
    dispatcher: Arc<PortDispatcher>,
    /// _multiplexer must be dropped first
    _multiplexer: multiplex::Multiplexer,
    _server: server::Server,
    client: client::Client,
}

impl Port {
    pub fn new<S: IpcSend + 'static, R: IpcRecv + 'static>(
        send: S,
        recv: R,
        dispatcher: Arc<PortDispatcher>,
        instance_key: InstanceKey,
        config: &FmlConfig,
    ) -> Self {
        let (mut multiplex_ends, _multiplexer) =
            multiplex::Multiplexer::create::<ServerOrClientForwarder, S, R>(send, recv, 2, 256);

        let client = {
            let (send, recv) = multiplex_ends.pop().unwrap();
            client::Client::new(send, recv, config.call_slots as u32)
        };

        let _server = {
            let (send, recv) = multiplex_ends.pop().unwrap();
            server::Server::new(dispatcher.clone(), send, recv, instance_key, config.server_threads, 128)
        };

        Port {
            dispatcher,
            _multiplexer,
            _server,
            client,
        }
    }

    pub fn call(&self, handle: ServiceObjectId, method: MethodId, data: Vec<u8>) -> Vec<u8> {
        self.client.call(handle, method, data)
    }

    pub fn delete(&self, handle: ServiceObjectId) {
        self.client.delete(handle);
    }

    pub fn dispatcher_get(&self) -> Arc<PortDispatcher> {
        self.dispatcher.clone()
    }
}
