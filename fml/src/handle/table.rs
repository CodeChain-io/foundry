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

use super::Service;
use crate::queue::Queue;
use std::sync::Arc;

#[cfg(debug_assertions)]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(1_000_000);
#[cfg(not(debug_assertions))]
const TIMEOUT: std::time::Duration = std::time::Duration::from_millis(50);

/// Per-port worst O(1) lookup table of service objects
pub struct ServiceObjectTable {
    handles: Vec<Option<Arc<dyn Service>>>,
    token: Queue<usize>,
}

impl ServiceObjectTable {
    pub fn new(size: usize) -> Self {
        let mut handles = Vec::new();
        let token = Queue::new(size);
        for i in 0..size {
            handles.push(None);
            token.push(i);
        }
        ServiceObjectTable {
            handles,
            token,
        }
    }

    pub fn create(&mut self, mut x: Arc<dyn Service>) -> Arc<dyn Service> {
        let token = self.token.pop(Some(TIMEOUT)).expect("Too many handle service object created");
        Arc::get_mut(&mut x).unwrap().get_handle_mut().id.index = token as u16;
        let slot = &mut self.handles[token];
        assert!(slot.is_none(), "ServiceObjectTable corrupted");
        *slot = Some(x.clone());
        x
    }

    pub fn remove(&mut self, token: usize) {
        let slot = &mut self.handles[token];
        assert!(slot.is_some(), "ServiceObjectTable corrupted");
        slot.take();
        self.token.push(token);
    }

    pub fn get(&self, token: usize) -> Arc<dyn Service> {
        self.handles[token].as_ref().unwrap().clone()
    }
}
