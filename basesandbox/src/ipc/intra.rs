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
use std::collections::hash_map::HashMap;
use std::sync::Mutex;

type RegisteredIpcEnds = (Sender<Vec<u8>>, Receiver<Vec<u8>>);

static POOL: OnceCell<Mutex<HashMap<String, RegisteredIpcEnds>>> = OnceCell::new();
fn get_pool_raw() -> &'static Mutex<HashMap<String, RegisteredIpcEnds>> {
    POOL.get_or_init(|| Mutex::new(HashMap::new()))
}

fn add_ends(key: String, ends: RegisteredIpcEnds) {
    assert!(get_pool_raw().lock().unwrap().insert(key, ends).is_none())
}

fn take_ends(key: &str) -> RegisteredIpcEnds {
    get_pool_raw().lock().unwrap().remove(key).unwrap()
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

        let (send1, recv1) = bounded(256);
        let (send2, recv2) = bounded(256);

        add_ends(key_server.clone(), (send1, recv2));
        add_ends(key_client.clone(), (send2, recv1));

        (serde_cbor::to_vec(&key_server).unwrap(), serde_cbor::to_vec(&key_client).unwrap())
    }

    type SendHalf = IntraSend;
    type RecvHalf = IntraRecv;

    fn new(data: Vec<u8>) -> Self {
        let key: String = serde_cbor::from_slice(&data).unwrap();
        let (send, recv) = take_ends(&key);
        Intra {
            send: IntraSend(send.clone()),
            recv: IntraRecv(recv, send),
        }
    }

    fn split(self) -> (Self::SendHalf, Self::RecvHalf) {
        (self.send, self.recv)
    }
}
