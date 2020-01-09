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
use crate::db::COL_PEER;
use cnetwork::{ManagingPeerdb, SocketAddr};
use kvdb::{DBTransaction, KeyValueDB};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
const COLUMN_TO_WRITE: Option<u32> = COL_PEER;

pub struct PeerDb {
    db: Arc<dyn KeyValueDB>,
}

impl PeerDb {
    pub fn new(database: Arc<dyn KeyValueDB>) -> Arc<Self> {
        Arc::new(Self {
            db: database,
        })
    }
}

impl ManagingPeerdb for PeerDb {
    fn insert(&self, key: &SocketAddr) {
        let mut batch = DBTransaction::new();
        let s = rlp::encode(key);
        let time: u64 = SystemTime::now().duration_since(UNIX_EPOCH).expect("There is no time machine.").as_secs();
        let value = rlp::encode(&time);
        batch.put(COLUMN_TO_WRITE, &s, &value);
        self.db.write(batch).expect("The key is not valid");
    }
    fn delete(&self, key: &SocketAddr) {
        let mut batch = DBTransaction::new();
        let s = rlp::encode(key);
        batch.delete(COLUMN_TO_WRITE, &s);
        self.db.write(batch).expect("The key is not valid");
    }
}
