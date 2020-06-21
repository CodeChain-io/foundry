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

use coordinator::context::{Context, MemPoolAccess, StateHistoryAccess, SubStorageAccess, Value};
use coordinator::Transaction;
use ctypes::{BlockId, TxHash};
use primitives::Bytes;
use std::collections::HashMap;

pub struct TestContext {
    tables: Vec<HashMap<Bytes, Option<Value>>>,
}

impl Context for TestContext {}

impl Default for TestContext {
    fn default() -> Self {
        Self {
            tables: vec![Default::default()],
        }
    }
}

fn insert_at_last(tables: &mut Vec<HashMap<Bytes, Option<Value>>>, key: Bytes, value: Option<Value>) {
    tables.last_mut().expect("The test context always have at least one table").insert(key, value);
}

impl SubStorageAccess for TestContext {
    fn get(&self, key: &dyn AsRef<[u8]>) -> Option<Value> {
        let key = key.as_ref().to_vec();
        self.tables.iter().rev().find_map(|table| table.get(&key).cloned()).flatten()
    }

    fn set(&mut self, key: &dyn AsRef<[u8]>, value: Value) {
        let key = key.as_ref().to_vec();
        insert_at_last(&mut self.tables, key, Some(value));
    }

    fn has(&self, key: &dyn AsRef<[u8]>) -> bool {
        let key = key.as_ref().to_vec();
        match self.tables.iter().rev().find_map(|table| table.get(&key)) {
            None => false,
            Some(&None) => false,
            Some(&Some(_)) => true,
        }
    }

    fn remove(&mut self, key: &dyn AsRef<[u8]>) {
        let key = key.as_ref().to_vec();
        insert_at_last(&mut self.tables, key, None);
    }

    fn create_checkpoint(&mut self) {
        self.tables.push(Default::default())
    }

    fn revert_to_the_checkpoint(&mut self) {
        self.tables.pop();
        assert!(!self.tables.is_empty());
    }

    fn discard_checkpoint(&mut self) {
        let last = self.tables.pop().expect("The test context always have at least one table");
        for (key, value) in last {
            insert_at_last(&mut self.tables, key, value);
        }
        assert!(!self.tables.is_empty());
    }
}

impl MemPoolAccess for TestContext {
    fn inject_transactions(&self, _txs: Vec<Transaction>) -> Vec<Result<TxHash, String>> {
        unimplemented!()
    }
}

impl StateHistoryAccess for TestContext {
    fn get_at(&self, _storage_id: u16, _block_number: Option<BlockId>, _key: &dyn AsRef<[u8]>) -> Option<Value> {
        unimplemented!()
    }

    fn has_at(&self, _storgae_id: u16, _block_number: Option<BlockId>, _key: &dyn AsRef<[u8]>) -> bool {
        unimplemented!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set() {
        let mut c = TestContext::default();
        let value1 = "b".to_string().into_bytes();
        c.set(&"a", value1.clone());
        assert_eq!(Some(value1), c.get(&"a"));
        let value2 = "c".to_string().into_bytes();
        c.set(&"a", value2.clone());
        assert_eq!(Some(value2), c.get(&"a"));
    }

    #[test]
    fn has() {
        let mut c = TestContext::default();
        let value1 = "b".to_string().into_bytes();
        assert!(!c.has(&"a"));
        c.set(&"a", value1);
        assert!(c.has(&"a"));
    }

    #[test]
    fn remove() {
        let mut c = TestContext::default();
        let value1 = "b".to_string().into_bytes();
        c.set(&"a", value1.clone());
        assert_eq!(Some(value1), c.get(&"a"));
        c.remove(&"a");
        assert_eq!(None, c.get(&"a"));
    }

    #[test]
    fn discard_checkpoint() {
        let mut c = TestContext::default();
        let value1 = "b".to_string().into_bytes();
        c.set(&"a", value1.clone());
        assert_eq!(Some(value1), c.get(&"a"));

        c.create_checkpoint();
        let value2 = "c".to_string().into_bytes();
        c.set(&"a", value2.clone());
        assert_eq!(Some(&value2), c.get(&"a").as_ref());

        c.discard_checkpoint();
        assert_eq!(Some(value2), c.get(&"a"));
    }

    #[test]
    fn revert_to_the_checkpoint() {
        let mut c = TestContext::default();
        let value1 = "b".to_string().into_bytes();
        c.set(&"a", value1.clone());
        assert_eq!(Some(&value1), c.get(&"a").as_ref());

        c.create_checkpoint();
        let value2 = "c".to_string().into_bytes();
        c.set(&"a", value2.clone());
        assert_eq!(Some(value2), c.get(&"a"));

        c.revert_to_the_checkpoint();
        assert_eq!(Some(value1), c.get(&"a"));
    }
}
