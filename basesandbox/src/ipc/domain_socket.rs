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
use std::sync::Arc;

// TODO: Use stream instead of datagram.
struct SocketInternal(UnixDatagram, String);

impl Drop for SocketInternal {
    fn drop(&mut self) {
        std::fs::remove_file(&self.1).unwrap();
    }
}

pub struct DomainSocketSend {
    socket: Arc<SocketInternal>,
}

impl IpcSend for DomainSocketSend {
    fn send(&self, data: &[u8]) {
        assert_eq!(self.socket.0.send(data).unwrap(), data.len());
    }
}

pub struct DomainSocketRecv {
    socket: Arc<SocketInternal>,
    buffer: RefCell<Vec<u8>>,
}

impl IpcRecv for DomainSocketRecv {
    type Terminator = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        self.socket.0.set_read_timeout(timeout).unwrap();
        let count = self.socket.0.recv(&mut self.buffer.borrow_mut()).unwrap();
        assert!(count < self.buffer.borrow().len(), "Unix datagram got data larger than the buffer.");
        if count == 0 {
            return Err(RecvError::Termination)
        }
        Ok(self.buffer.borrow()[0..count].to_vec())
    }

    fn create_terminator(&self) -> Self::Terminator {
        Terminator(self.socket.clone())
    }
}

pub struct Terminator(Arc<SocketInternal>);

impl Terminate for Terminator {
    fn terminate(&self) {
        if let Err(e) = (self.0).0.shutdown(std::net::Shutdown::Both) {
            assert_eq!(e.kind(), std::io::ErrorKind::NotConnected);
        }
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
            serde_cbor::to_vec(&(true, &address_server, &address_client)).unwrap(),
            serde_cbor::to_vec(&(false, &address_client, &address_server)).unwrap(),
        )
    }

    type SendHalf = DomainSocketSend;
    type RecvHalf = DomainSocketRecv;

    fn new(data: Vec<u8>) -> Self {
        let (am_i_server, address_src, address_dst): (bool, String, String) = serde_cbor::from_slice(&data).unwrap();
        let socket = UnixDatagram::bind(&address_src).unwrap();
        let mut success = false;

        for _ in 0..100 {
            if socket.connect(&address_dst).is_ok() {
                success = true;
                break
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
        assert!(success, "Failed to establish domain socket");
        socket.set_write_timeout(None).unwrap();
        socket.set_nonblocking(false).unwrap();

        // Handshake
        if am_i_server {
            socket.set_read_timeout(Some(std::time::Duration::from_millis(100))).unwrap();
            let mut buf = [0 as u8; 32];
            let count = socket.recv(&mut buf).unwrap();
            assert_eq!(count, 3);
            assert_eq!(&buf[0..3], b"hey");
            socket.send(b"hello").unwrap();
            let count = socket.recv(&mut buf).unwrap();
            assert_eq!(count, 2);
            assert_eq!(&buf[0..2], b"hi");
        } else {
            socket.send(b"hey").unwrap();
            socket.set_read_timeout(Some(std::time::Duration::from_millis(100))).unwrap();
            let mut buf = [0 as u8; 32];
            let count = socket.recv(&mut buf).unwrap();
            assert_eq!(count, 5);
            assert_eq!(&buf[0..5], b"hello");
            socket.send(b"hi").unwrap();
        }

        let socket = Arc::new(SocketInternal(socket, address_src));
        DomainSocket {
            send: DomainSocketSend {
                socket: socket.clone(),
            },
            recv: DomainSocketRecv {
                socket,
                buffer: RefCell::new(vec![0; 1024 * 8 + 1]),
            },
        }
    }

    fn split(self) -> (Self::SendHalf, Self::RecvHalf) {
        (self.send, self.recv)
    }
}
