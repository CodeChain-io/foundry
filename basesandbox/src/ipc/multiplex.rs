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

use super::{IpcRecv, IpcSend, RecvError, Terminate};
use crossbeam::channel::bounded;
use parking_lot::Mutex;
use std::thread;

type Sender = crossbeam::channel::Sender<Vec<u8>>;
type Receiver = crossbeam::channel::Receiver<Vec<u8>>;

pub trait Forward {
    fn forward(data: &[u8]) -> usize;
}

fn sender<S: IpcSend>(send: S, recv: Receiver) {
    loop {
        let data = recv.recv().unwrap();
        // special exit flag
        if data.is_empty() {
            break
        }
        send.send(&data);
    }
}

fn receiver<F: Forward, R: IpcRecv>(recv: R, send: Vec<Sender>) {
    loop {
        let data = match recv.recv(None) {
            Err(RecvError::TimeOut) => panic!(),
            Err(RecvError::Termination) => return,
            Ok(x) => x,
        };
        send[F::forward(&data)].send(data).unwrap();
    }
}

pub struct Multiplexer {
    sender_thread: Option<thread::JoinHandle<()>>,
    receiver_thread: Option<thread::JoinHandle<()>>,
    /// Here Mutex is used to make the Multiplxer Sync, while dyn Terminate isn't.
    termiantor: Option<Mutex<Box<dyn Terminate>>>,
    sender_send: Sender, // will convey the flag of termination
}

impl Multiplexer {
    pub fn create<F: Forward, S: IpcSend + 'static, R: IpcRecv + 'static>(
        ipc_send: S,
        ipc_recv: R,
        multiplex: usize,
        channel_capacity: usize,
    ) -> (Vec<(Sender, Receiver)>, Self) {
        let (sender_send, sender_recv) = bounded(channel_capacity);
        let mut receiver_sends = Vec::<Sender>::new();
        let mut channel_ends = Vec::<(Sender, Receiver)>::new();
        let termiantor: Option<Mutex<Box<dyn Terminate>>> = Some(Mutex::new(Box::new(ipc_recv.create_terminator())));

        for _ in 0..multiplex {
            let (send, recv) = bounded(channel_capacity);
            channel_ends.push((sender_send.clone(), recv));
            receiver_sends.push(send);
        }

        let sender_thread = thread::spawn(move || {
            sender(ipc_send, sender_recv);
        });

        let receiver_thread = thread::spawn(move || {
            receiver::<F, R>(ipc_recv, receiver_sends);
        });

        (channel_ends, Multiplexer {
            sender_thread: Some(sender_thread),
            receiver_thread: Some(receiver_thread),
            termiantor,
            sender_send,
        })
    }
}

impl Drop for Multiplexer {
    fn drop(&mut self) {
        self.termiantor.take().unwrap().into_inner().terminate();
        self.sender_send.send(Vec::new()).unwrap();
        self.receiver_thread.take().unwrap().join().unwrap();
        self.sender_thread.take().unwrap().join().unwrap();
    }
}
