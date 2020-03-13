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

use crate::cmodule::link::{Linkable, Linker, Port, Result};
use crossbeam_channel::{Receiver, Sender};
use std::path::Path;
use std::sync::Arc;

pub struct BaseHandle {
    // that of BasePort::handles
    index: usize,
}

impl BaseHandle {
    pub fn new(index: usize) -> Self {
        BaseHandle {
            index,
        }
    }
}

trait MyTest: Send + Sync {}

pub struct BasePort {
    sender: Option<Sender<Vec<u8>>>,
    receiver: Option<Receiver<Vec<u8>>>,
    handles: Vec<Option<BaseHandle>>,
}

impl BasePort {
    pub fn new(max_handles: usize) -> Self {
        let mut empty_handles: Vec<Option<BaseHandle>> = Vec::new();
        for _ in 0..max_handles {
            empty_handles.push(None);
        }
        BasePort {
            sender: None,
            receiver: None,
            handles: empty_handles,
        }
    }

    pub fn send(&self, data: Vec<u8>) {
        self.sender.as_ref().unwrap().send(data).unwrap();
    }

    pub fn recv(&self) -> Vec<u8> {
        self.receiver.as_ref().unwrap().recv().unwrap()
    }
}

impl Port for BasePort {
    fn send(&mut self, desc: &[u8]) -> &mut dyn Port {
        let sender = self.sender.as_ref().expect("Port is not linked");
        let encoded = b"This must be encoding of handle_slot.unwrap()".to_vec();

        for (i, handle_slot) in self.handles.iter_mut().enumerate() {
            if handle_slot.is_none() {
                *handle_slot = Some(BaseHandle::new(i));
                sender.send(encoded).unwrap();
                return self
            }
        }
        panic!("No availiable handle slot");
    }

    fn receive(&mut self, slots: &[&str]) -> &mut dyn Port {
        // TODO
        self
    }

    fn link(&mut self, sender: Sender<Vec<u8>>, receiver: Receiver<Vec<u8>>) {
        self.sender = Some(sender);
        self.receiver = Some(receiver);
    }
}
