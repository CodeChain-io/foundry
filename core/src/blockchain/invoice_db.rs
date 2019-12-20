// Copyright 2018-2019 Kodebox, Inc.
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

use std::collections::HashMap;
use std::sync::Arc;

use ctypes::TxHash;
use kvdb::{DBTransaction, KeyValueDB};
use parking_lot::RwLock;
use primitives::{H256, H264};

use crate::db::{self, CacheUpdatePolicy, Key, Readable, Writable};

/// Structure providing fast access to blockchain data.
///
/// **Does not do input data verification.**
pub struct InvoiceDB {
    // transaction hash -> error hint
    hash_cache: RwLock<HashMap<TxHash, Option<String>>>,

    db: Arc<dyn KeyValueDB>,
}

impl InvoiceDB {
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
    pub fn insert_invoice(&self, batch: &mut DBTransaction, hash: TxHash, error_hint: Option<String>) {
        if self.is_known_error_hint(&hash) {
            return
        }

        let mut hint_cache = self.hash_cache.write();

        batch.write_with_cache(db::COL_ERROR_HINT, &mut *hint_cache, hash, error_hint, CacheUpdatePolicy::Remove);
    }
}

/// Interface for querying invoices.
pub trait InvoiceProvider {
    /// Returns true if invoices for given hash is known
    fn is_known_error_hint(&self, hash: &TxHash) -> bool;

    /// Get error hint
    fn error_hint(&self, hash: &TxHash) -> Option<String>;
}

impl InvoiceProvider for InvoiceDB {
    fn is_known_error_hint(&self, hash: &TxHash) -> bool {
        self.db.exists_with_cache(db::COL_ERROR_HINT, &self.hash_cache, hash)
    }

    fn error_hint(&self, hash: &TxHash) -> Option<String> {
        self.db.read_with_cache(db::COL_ERROR_HINT, &mut *self.hash_cache.write(), hash)?
    }
}

enum ErrorHintIndex {
    HashToHint = 1,
}

impl From<ErrorHintIndex> for u8 {
    fn from(e: ErrorHintIndex) -> Self {
        e as Self
    }
}

impl Key<Option<String>> for TxHash {
    type Target = H264;

    fn key(&self) -> H264 {
        with_index(self, ErrorHintIndex::HashToHint)
    }
}

fn with_index(hash: &H256, i: ErrorHintIndex) -> H264 {
    let mut result = H264::default();
    result[0] = i as u8;
    (*result)[1..].copy_from_slice(hash);
    result
}
