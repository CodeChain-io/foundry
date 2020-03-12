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
