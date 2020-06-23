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

use crate::Transaction;
use ctypes::TxHash;
pub use ctypes::{BlockId, Header, StorageId};

/// A `Context` provides the interface against the system services such as moulde substorage access,
/// mempool access and state history access.
pub trait Context: SubStorageAccess + MemPoolAccess + StateHistoryAccess {}

pub type Key = dyn AsRef<[u8]>;
pub type Value = Vec<u8>;

// Interface between each module and the coordinator
pub trait SubStorageAccess: Send + Sync {
    fn get(&self, key: &Key) -> Option<Value>;
    fn set(&self, key: &Key, value: Value);
    fn has(&self, key: &Key) -> bool;
    fn remove(&self, key: &Key);

    /// Create a recoverable checkpoint of this state
    fn create_checkpoint(&self);
    /// Revert to the last checkpoint and discard it
    fn revert_to_the_checkpoint(&self);
    /// Merge last checkpoint with the previous
    fn discard_checkpoint(&self);
}

// Interface between host and the coordinator
pub trait StorageAccess {
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

pub trait MemPoolAccess {
    fn inject_transactions(&self, txs: Vec<Transaction>) -> Vec<Result<TxHash, String>>;
}

pub trait ChainHistoryAccess {
    fn get_block_header(&self, block_id: BlockId) -> Option<Header>;
}

// Interface observable from modules
pub trait SubStateHistoryAccess {
    fn get_at(&self, block_number: Option<BlockId>, key: &Key) -> Option<Value>;
    fn has_at(&self, block_number: Option<BlockId>, key: &Key) -> bool;
}

// Host side internal trait
pub trait StateHistoryAccess {
    fn get_at(&self, storage_id: StorageId, block_number: Option<BlockId>, key: &Key) -> Option<Value>;
    fn has_at(&self, storgae_id: StorageId, block_number: Option<BlockId>, key: &Key) -> bool;
}
