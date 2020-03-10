// Copyright 2019 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::mem_pool_types::MemPoolItem;
use crate::db as dblib;
use crate::error::Error;
use crate::transaction::UnverifiedTransaction;
use coordinator::TxOrigin;
use ctypes::BlockNumber;
use kvdb::{DBTransaction, KeyValueDB};
use primitives::H256;
use rlp::Encodable;
use std::collections::HashMap;
use std::convert::{TryFrom, TryInto};

pub type PoolingInstant = BlockNumber;

#[derive(Clone, Eq, PartialEq, Debug, RlpEncodable, RlpDecodable)]
pub struct MemPoolItemProjection {
    /// Transaction.
    pub tx: UnverifiedTransaction,
    /// Transaction origin.
    pub origin: TxOrigin,
    /// Insertion time
    pub inserted_block_number: PoolingInstant,
    /// Insertion timstamp
    pub inserted_timestamp: u64,
    /// ID assigned upon insertion, should be unique.
    pub insertion_id: u64,
}

impl TryFrom<MemPoolItemProjection> for MemPoolItem {
    type Error = Error;
    fn try_from(mem_pool: MemPoolItemProjection) -> Result<Self, Error> {
        let verified_tx = mem_pool.tx.try_into()?;
        Ok(MemPoolItem::new(
            verified_tx,
            mem_pool.origin,
            mem_pool.inserted_block_number,
            mem_pool.inserted_timestamp,
            mem_pool.insertion_id,
        ))
    }
}

const PREFIX_SIZE: usize = 5;
const PREFIX_ITEM: &[u8; PREFIX_SIZE] = b"item_";

pub fn backup_batch_with_capacity(length: usize) -> DBTransaction {
    DBTransaction::with_capacity(length)
}

pub fn backup_item(batch: &mut DBTransaction, key: H256, item: &MemPoolItem) {
    let mut db_key = PREFIX_ITEM.to_vec();
    db_key.extend_from_slice(key.as_ref());
    batch.put(dblib::COL_MEMPOOL, db_key.as_ref(), item.rlp_bytes().as_ref());
}

pub fn remove_item(batch: &mut DBTransaction, key: &H256) {
    let mut db_key = PREFIX_ITEM.to_vec();
    db_key.extend_from_slice(key.as_ref());
    batch.delete(dblib::COL_MEMPOOL, db_key.as_ref());
}

pub fn recover_to_data(db: &dyn KeyValueDB) -> HashMap<H256, MemPoolItem> {
    let mut by_hash = HashMap::new();

    for (key, value) in db.iter(dblib::COL_MEMPOOL) {
        let bytes = (*value).to_vec();
        let rlp = rlp::Rlp::new(&bytes);
        let decoded_key = (key.as_ref()[PREFIX_SIZE..]).into();
        let mem_pool_projection: MemPoolItemProjection = rlp.as_val().unwrap();
        let decoded_item = mem_pool_projection.try_into().expect("DB corruption detected");
        by_hash.insert(decoded_key, decoded_item);
    }

    by_hash
}
