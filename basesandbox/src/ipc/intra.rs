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
use crossbeam::channel::{bounded, Receiver, RecvTimeoutError, Sender};
use once_cell::sync::OnceCell;
use parking_lot::Mutex;
use std::collections::hash_map::HashMap;

struct RegisteredIpcEnds {
    is_server: bool,
    send: Sender<Vec<u8>>,
    recv: Receiver<Vec<u8>>,
    /// One copy of Counterparty's Send end
    send_for_termination: Sender<Vec<u8>>,
}

static POOL: OnceCell<Mutex<HashMap<String, RegisteredIpcEnds>>> = OnceCell::new();
fn get_pool_raw() -> &'static Mutex<HashMap<String, RegisteredIpcEnds>> {
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

fn add_ends(key: String, ends: RegisteredIpcEnds) {
    assert!(get_pool_raw().lock().insert(key, ends).is_none())
}

fn take_ends(key: &str) -> RegisteredIpcEnds {
    get_pool_raw().lock().remove(key).unwrap()
}

pub struct IntraSend(Sender<Vec<u8>>);

impl IpcSend for IntraSend {
    fn send(&self, data: &[u8]) {
        self.0.send(data.to_vec()).unwrap()
    }
}

pub struct IntraRecv(Receiver<Vec<u8>>, Sender<Vec<u8>>);

pub struct Terminator(Sender<Vec<u8>>);

impl Terminate for Terminator {
    fn terminate(&self) {
        self.0.send([].to_vec()).unwrap();
    }
}

impl IpcRecv for IntraRecv {
    type Terminator = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        let x = if let Some(t) = timeout {
            self.0.recv_timeout(t).map_err(|e| {
                if e == RecvTimeoutError::Timeout {
                    RecvError::TimeOut
                } else {
                    panic!()
                }
            })
        } else {
            Ok(self.0.recv().unwrap())
        }?;

        if x.is_empty() {
            return Err(RecvError::Termination)
        }
        Ok(x)
    }

    fn create_terminator(&self) -> Self::Terminator {
        Terminator(self.1.clone())
    }
}

/// This acts like an IPC, but is actually an intra-process communication.
/// It will be useful when you have to simulate IPC, but the two ends don't have
/// to be actually in separated processes.
pub struct Intra {
    send: IntraSend,
    recv: IntraRecv,
}

impl IpcSend for Intra {
    fn send(&self, data: &[u8]) {
        self.send.send(data)
    }
}

impl IpcRecv for Intra {
    type Terminator = Terminator;

    fn recv(&self, timeout: Option<std::time::Duration>) -> Result<Vec<u8>, RecvError> {
        self.recv.recv(timeout)
    }

    fn create_terminator(&self) -> Self::Terminator {
        self.recv.create_terminator()
    }
}

impl Ipc for Intra {
    fn arguments_for_both_ends() -> (Vec<u8>, Vec<u8>) {
        let key_server = generate_random_name();
        let key_client = generate_random_name();

        let (send_server, recv_client) = bounded(256);
        let (send_client, recv_server) = bounded(256);

        add_ends(key_server.clone(), RegisteredIpcEnds {
            is_server: true,
            send: send_server.clone(),
            recv: recv_server,
            send_for_termination: send_client.clone(),
        });
        add_ends(key_client.clone(), RegisteredIpcEnds {
            is_server: false,
            send: send_client,
            recv: recv_client,
            send_for_termination: send_server,
        });

        (serde_cbor::to_vec(&key_server).unwrap(), serde_cbor::to_vec(&key_client).unwrap())
    }

    type SendHalf = IntraSend;
    type RecvHalf = IntraRecv;

    fn new(data: Vec<u8>) -> Self {
        let key: String = serde_cbor::from_slice(&data).unwrap();
        let RegisteredIpcEnds {
            is_server,
            send,
            recv,
            send_for_termination,
        } = take_ends(&key);

        // Handshake
        let timeout = std::time::Duration::from_millis(1000);
        if is_server {
            let x = recv.recv_timeout(timeout).unwrap();
            assert_eq!(x, b"hey");
            send.send(b"hello".to_vec()).unwrap();
            let x = recv.recv_timeout(timeout).unwrap();
            assert_eq!(x, b"hi");
        } else {
            send.send(b"hey".to_vec()).unwrap();
            let x = recv.recv().unwrap();
            assert_eq!(x, b"hello");
            send.send(b"hi".to_vec()).unwrap();
        }

        Intra {
            send: IntraSend(send),
            recv: IntraRecv(recv, send_for_termination),
        }
    }

    fn split(self) -> (Self::SendHalf, Self::RecvHalf) {
        (self.send, self.recv)
    }
}
