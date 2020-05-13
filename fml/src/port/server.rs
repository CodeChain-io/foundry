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

use super::PacketHeader;
use super::{DELETE_INDICATOR, SLOT_CALL_OR_RETURN_INDICATOR};
use crate::context::single_process_support;
use crate::handle::{dispatch::delete, PortDispatcher};
use crate::queue::Queue;
use crossbeam::channel::{bounded, Receiver, Sender};
use std::io::Cursor;
use std::sync::Arc;
use std::thread;

#[cfg(debug_assertions)]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1_000_000);
#[cfg(not(debug_assertions))]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

fn service_handler(
    invoke: Receiver<Vec<u8>>,
    response: Sender<Vec<u8>>,
    dispatcher: Arc<PortDispatcher>,
    instance_key: single_process_support::InstanceKey,
    token: u32,
    token_queue: Arc<Queue<u32>>,
) -> Result<(), ()> {
    // This setup makes a thread local unique key for each instance of module
    // so that the service can retrieve its own global context.
    single_process_support::set_key(instance_key);
    loop {
        let data = invoke.recv().map_err(|_| ())?;
        if data.len() < std::mem::size_of::<PacketHeader>() {
            panic!("Invalid packet received: {:?}", data);
        }
        let mut header = PacketHeader::new(&data);
        header.slot -= SLOT_CALL_OR_RETURN_INDICATOR;
        let mut buffer: Vec<u8> = vec![0; std::mem::size_of::<PacketHeader>()];

        if header.method == DELETE_INDICATOR {
            delete(dispatcher.get_id(), header.handle);
        } else {
            dispatcher.dispatch(header.handle, header.method, &data, {
                let mut c = Cursor::new(&mut buffer);
                c.set_position(std::mem::size_of::<PacketHeader>() as u64);
                c
            });
        }
        header.write(&mut buffer);
        response.send(buffer).unwrap();
        token_queue.push(token);
    }
}

fn receiver(
    ipc_send: Sender<Vec<u8>>,
    ipc_recv: Receiver<Vec<u8>>,
    dispatcher: Arc<PortDispatcher>,
    instance_key: single_process_support::InstanceKey,
    max_threads: usize,
    channel_capcity: usize,
) {
    // Handling service with threads is just receiver()'s implementation detail.
    // So all these thread management stuffs belong here, not the Server.
    let mut invocation_send: Vec<Sender<Vec<u8>>> = Vec::new();
    let mut service_handlers: Vec<thread::JoinHandle<()>> = Vec::new();
    let token_queue = Arc::new(Queue::<u32>::new(max_threads));

    for i in 0..max_threads {
        let (send, recv) = bounded(channel_capcity);
        invocation_send.push(send);
        let dispatcher_ = dispatcher.clone();
        let ipc_send_ = ipc_send.clone();
        let token_queue_ = token_queue.clone();
        service_handlers.push(thread::spawn(move || {
            service_handler(recv, ipc_send_, dispatcher_, instance_key, i as u32, token_queue_).ok();
        }));
        token_queue.push(i as u32);
    }

    while let Ok(data) = ipc_recv.recv() {
        invocation_send[token_queue.pop(Some(TIMEOUT)).expect("Servcie handler unavailiable") as usize]
            .send(data)
            .unwrap();
    }

    drop(invocation_send);
    while let Some(x) = service_handlers.pop() {
        x.join().unwrap();
    }
}

pub struct Server {
    receiver_thread: Option<thread::JoinHandle<()>>,
}

impl Server {
    pub fn new(
        dispatcher: Arc<PortDispatcher>,
        ipc_send: Sender<Vec<u8>>,
        ipc_recv: Receiver<Vec<u8>>,
        instance_key: single_process_support::InstanceKey,
        max_threads: usize,
        channel_capcity: usize,
    ) -> Self {
        Server {
            receiver_thread: Some(thread::spawn(move || {
                receiver(ipc_send, ipc_recv, dispatcher, instance_key, max_threads, channel_capcity)
            })),
        }
    }
}

impl Drop for Server {
    fn drop(&mut self) {
        self.receiver_thread.take().unwrap().join().unwrap();
    }
}
