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

use crate::context::{StorageAccess, SubStorageAccess};
use ctypes::StorageId;
use parking_lot::Mutex;
use remote_trait_object::Service;
use std::sync::Arc;

pub struct SubStorageView {
    id: StorageId,
    storage: Arc<Mutex<dyn StorageAccess>>,
}

unsafe impl Send for SubStorageView {}
unsafe impl Sync for SubStorageView {}

impl SubStorageView {
    pub fn of(id: StorageId, storage: Arc<Mutex<dyn StorageAccess>>) -> SubStorageView {
        SubStorageView {
            id,
            storage,
        }
    }
}

struct ToAsRef<'a>(&'a [u8]);

impl<'a> AsRef<[u8]> for ToAsRef<'a> {
    fn as_ref(&self) -> &[u8] {
        self.0
    }
}

impl Service for SubStorageView {}

impl SubStorageAccess for SubStorageView {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.lock().get(self.id, &ToAsRef(key))
    }

    fn set(&mut self, key: &[u8], value: Vec<u8>) {
        self.storage.lock().set(self.id, &ToAsRef(key), value)
    }

    fn has(&self, key: &[u8]) -> bool {
        self.storage.lock().has(self.id, &ToAsRef(key))
    }

    fn remove(&mut self, key: &[u8]) {
        self.storage.lock().remove(self.id, &ToAsRef(key))
    }

    fn create_checkpoint(&mut self) {
        self.storage.lock().create_checkpoint()
    }

    fn revert_to_the_checkpoint(&mut self) {
        self.storage.lock().create_checkpoint()
    }

    fn discard_checkpoint(&mut self) {
        self.storage.lock().discard_checkpoint()
    }
}
