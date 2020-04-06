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

use super::*;
use std::cell::RefCell;
use std::os::unix::net::UnixDatagram;
use std::path::Path;
use std::sync::Arc;

// TODO: Use stream instead of datagram.
struct SocketInternal(UnixDatagram, String);

impl Drop for SocketInternal {
    fn drop(&mut self) {
        self.0.shutdown(std::net::Shutdown::Both).unwrap();
        std::fs::remove_file(&self.1).unwrap();
    }
}

pub struct DomainSocketSend {
    address_dst: String,
    socket: Arc<SocketInternal>,
}

impl IpcSend for DomainSocketSend {
    fn send(&self, data: &[u8]) {
        assert_eq!(self.socket.0.send_to(data, &self.address_dst).unwrap(), data.len());
    }
}

pub struct DomainSocketRecv {
    address_dst: String,
    socket: Arc<SocketInternal>,
    buffer: RefCell<Vec<u8>>,
}

impl IpcRecv for DomainSocketRecv {
    type Terminator = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        self.socket.0.set_read_timeout(timeout).unwrap();
        let (count, address) = self.socket.0.recv_from(&mut self.buffer.borrow_mut()).unwrap();
        assert!(count < self.buffer.borrow().len(), "Unix datagram got data larger than the buffer.");
        if count == 0 {
            return Err(RecvError::Termination)
        }
        assert_eq!(
            address.as_pathname().unwrap(),
            Path::new(&self.address_dst),
            "Unix datagram received packet from an unexpected sender."
        );
        Ok(self.buffer.borrow()[0..count].to_vec())
    }

    fn create_terminator(&self) -> Self::Terminator {
        Terminator(self.socket.clone())
    }
}

pub struct Terminator(Arc<SocketInternal>);

impl Terminate for Terminator {
    fn terminate(&self) {
        (self.0).0.shutdown(std::net::Shutdown::Both).unwrap();
    }
}

pub struct DomainSocket {
    send: DomainSocketSend,
    recv: DomainSocketRecv,
}

impl IpcSend for DomainSocket {
    fn send(&self, data: &[u8]) {
        self.send.send(data)
    }
}

impl IpcRecv for DomainSocket {
    type Terminator = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        self.recv.recv(timeout)
    }

    fn create_terminator(&self) -> Self::Terminator {
        self.recv.create_terminator()
    }
}

impl Ipc for DomainSocket {
    fn arguments_for_both_ends() -> (Vec<u8>, Vec<u8>) {
        let address_server = format!("{}/{}", std::env::temp_dir().to_str().unwrap(), generate_random_name());
        let address_client = format!("{}/{}", std::env::temp_dir().to_str().unwrap(), generate_random_name());
        (
            serde_cbor::to_vec(&(&address_server, &address_client)).unwrap(),
            serde_cbor::to_vec(&(&address_client, &address_server)).unwrap(),
        )
    }

    type SendHalf = DomainSocketSend;
    type RecvHalf = DomainSocketRecv;

    fn new(data: Vec<u8>) -> Self {
        let (address_src, address_dst): (String, String) = serde_cbor::from_slice(&data).unwrap();
        let socket = Arc::new(SocketInternal(UnixDatagram::bind(&address_src).unwrap(), address_src));
        DomainSocket {
            send: DomainSocketSend {
                address_dst: address_dst.clone(),
                socket: socket.clone(),
            },
            recv: DomainSocketRecv {
                address_dst,
                socket,
                buffer: RefCell::new(vec![0; 1024 * 8 + 1]),
            },
        }
    }

    fn split(self) -> (Self::SendHalf, Self::RecvHalf) {
        (self.send, self.recv)
    }
}
