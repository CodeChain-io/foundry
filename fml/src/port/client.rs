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
use crate::handle::{MethodId, ServiceObjectId};
use crate::queue::Queue;
use crossbeam::channel::{bounded, Receiver, Sender};
use std::sync::Arc;
use std::thread;

use super::SlotId;

#[cfg(debug_assertions)]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1_000_000);
#[cfg(not(debug_assertions))]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

/// CallSlot represents an instance of call to the another module
struct CallSlot {
    id: SlotId,
    invoke: Sender<Vec<u8>>,
    response: Receiver<Vec<u8>>,
}

fn receiver(recv: Receiver<Vec<u8>>, response_send: Vec<Sender<Vec<u8>>>) -> Result<(), ()> {
    loop {
        let data = recv.recv().map_err(|_| ())?;
        let header = PacketHeader::new(&data);
        response_send[header.slot as usize].send(data).unwrap();
    }
}

pub struct Client {
    call_slots: Arc<Queue<CallSlot>>,
    receiver_thread: Option<thread::JoinHandle<()>>,
}

impl Client {
    pub fn new(ipc_send: Sender<Vec<u8>>, ipc_recv: Receiver<Vec<u8>>, callslot_size: SlotId) -> Self {
        let call_slots = Arc::new(Queue::new(callslot_size as usize));
        let mut response_send = Vec::new();
        for i in 0..callslot_size {
            let (send_slot, recv_slot) = bounded(1);
            call_slots.push(CallSlot {
                id: i,
                invoke: ipc_send.clone(),
                response: recv_slot,
            });
            response_send.push(send_slot);
        }

        Client {
            call_slots,
            receiver_thread: Some(thread::spawn(move || {
                receiver(ipc_recv, response_send).ok();
            })),
        }
    }

    /// Caller must have reserved sizeof(PacketHeader) bytes on the first of data
    pub fn call(&self, handle: ServiceObjectId, method: MethodId, mut data: Vec<u8>) -> Vec<u8> {
        let slot = self.call_slots.pop(Some(TIMEOUT)).expect("Module doesn't respond");
        let header = PacketHeader {
            handle,
            method,
            slot: slot.id as u32 + SLOT_CALL_OR_RETURN_INDICATOR,
        };
        header.write(&mut data);
        slot.invoke.send(data).unwrap();
        let return_value = slot.response.recv().unwrap();
        self.call_slots.push(slot); //return back
        return_value
    }

    /// request to delete given handle from the registry of exporter
    pub fn delete(&self, handle: ServiceObjectId) {
        let slot = self.call_slots.pop(Some(TIMEOUT)).expect("Module doesn't respond");
        let mut buffer = vec![0 as u8; std::mem::size_of::<PacketHeader>()];
        let header = PacketHeader {
            handle,
            method: DELETE_INDICATOR,
            slot: slot.id as u32 + SLOT_CALL_OR_RETURN_INDICATOR,
        };
        header.write(&mut buffer);
        slot.invoke.send(buffer).unwrap();
        let return_value = slot.response.recv().unwrap();
        assert_eq!(PacketHeader::new(&return_value).method, DELETE_INDICATOR);
        self.call_slots.push(slot) //return back
    }
}

impl Drop for Client {
    fn drop(&mut self) {
        self.receiver_thread.take().unwrap().join().unwrap();
    }
}
