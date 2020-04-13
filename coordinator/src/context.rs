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

pub use ctypes::StorageId;

/// A `Context` provides the interface against the system services such as DB access,
pub trait Context: SubStorageAccess {}

pub type Key = dyn AsRef<[u8]>;
pub type Value = Vec<u8>;
pub trait SubStorageAccess {
    fn get(&self, storage_id: StorageId, key: &Key) -> Option<Value>;
    fn set(&mut self, storage_id: StorageId, key: &Key, value: Value);
    fn has(&self, storage_id: StorageId, key: &Key) -> bool;
    fn remove(&mut self, storage_id: StorageId, key: &Key);

    /// Create a recoverable checkpoint of this state
    fn create_checkpoint(&mut self);
    /// Revert to the last checkpoint and discard it
    fn revert_to_the_checkpoint(&mut self);
    /// Merge last checkpoint with the previous
    fn discard_checkpoint(&mut self);
}
