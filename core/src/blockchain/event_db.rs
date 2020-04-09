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

use crate::db::{self, CacheUpdatePolicy, Readable, Writable};
use crate::event::{EventSource, Events};
use coordinator::types::Event;
use kvdb::{DBTransaction, KeyValueDB};
use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

pub struct EventDB {
    hash_cache: RwLock<HashMap<EventSource, Events>>,
    db: Arc<dyn KeyValueDB>,
}

impl EventDB {
    /// Create new instance of blockchain from given Genesis.
    pub fn new(db: Arc<dyn KeyValueDB>) -> Self {
        Self {
            hash_cache: Default::default(),
            db,
        }
    }

    /// Inserts the block into backing cache database.
    /// Expects the block to be valid and already verified.
    /// If the block is already known, does nothing.
    pub fn insert_events(&self, batch: &mut DBTransaction, source: EventSource, events: Vec<Event>) {
        if self.is_known_source(&source) {
            return
        }

        let mut cache = self.hash_cache.write();
        batch.write_with_cache(db::COL_EVENT, &mut *cache, source, Events(events), CacheUpdatePolicy::Remove);
    }
}

/// Interface for querying events.
pub trait EventProvider {
    fn is_known_source(&self, source: &EventSource) -> bool;

    fn events(&self, source: &EventSource) -> Vec<Event>;
}

impl EventProvider for EventDB {
    fn is_known_source(&self, source: &EventSource) -> bool {
        self.db.exists_with_cache(db::COL_EVENT, &self.hash_cache, source)
    }

    fn events(&self, source: &EventSource) -> Vec<Event> {
        self.db.read_with_cache(db::COL_EVENT, &mut *self.hash_cache.write(), source).unwrap_or_default().0
    }
}

#[cfg(test)]
mod tests {
    use ctypes::TxHash;

    use super::*;

    #[test]
    fn insert_and_check_events() {
        let db = Arc::new(kvdb_memorydb::create(crate::db::NUM_COLUMNS.unwrap_or(0)));
        let event_db = EventDB::new(db.clone());

        let source = EventSource::Transaction(TxHash::default());

        let event1 = Event {
            key: "key1".to_string(),
            value: vec![1, 2, 3, 4, 5],
        };
        let event2 = Event {
            key: "key2".to_string(),
            value: vec![2, 3, 4, 5],
        };
        let events = vec![event1, event2];

        let mut batch = DBTransaction::new();
        event_db.insert_events(&mut batch, source.clone(), events.clone());
        db.write_buffered(batch);

        assert!(event_db.is_known_source(&source));
        assert_eq!(event_db.events(&source), events);
    }
}
