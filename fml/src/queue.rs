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

use crossbeam::channel::{bounded, Receiver, Sender};
use std::sync::{Arc, Mutex};

/// Blocking concurrent Queue. (Crossbeam's queue doens't block)
pub struct Queue<T> {
    sender: Arc<Mutex<Sender<T>>>,
    recver: Arc<Mutex<Receiver<T>>>,
}

impl<T> Queue<T> {
    pub fn new(size: usize) -> Self {
        let (sender, recver) = bounded(size);
        Queue {
            sender: Arc::new(Mutex::new(sender)),
            recver: Arc::new(Mutex::new(recver)),
        }
    }

    pub fn push(&self, x: T) {
        self.sender.lock().unwrap().send(x).unwrap();
    }

    pub fn pop(&self, timeout: Option<std::time::Duration>) -> Result<T, ()> {
        if let Some(duration) = timeout {
            self.recver.lock().unwrap().recv_timeout(duration).map_err(|_| ())
        } else {
            self.recver.lock().unwrap().recv().map_err(|_| ())
        }
    }
}
