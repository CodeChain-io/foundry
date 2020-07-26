// Copyright 2019-2020 Kodebox, Inc.
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

use crate::transaction::VerifiedTransaction;
use ctypes::{BlockNumber, TxHash};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::cmp::Ordering;
use std::collections::BTreeSet;

/// Point in time when transaction was inserted.
pub type PoolingInstant = BlockNumber;

/// Transaction origin
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TxOrigin {
    /// Transaction coming from local RPC
    Local,
    /// External transaction received from network
    External,
}

type TxOriginType = u8;
const LOCAL: TxOriginType = 0x01;
const EXTERNAL: TxOriginType = 0x02;

impl Encodable for TxOrigin {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            TxOrigin::Local => LOCAL.rlp_append(s),
            TxOrigin::External => EXTERNAL.rlp_append(s),
        };
    }
}

impl Decodable for TxOrigin {
    fn decode(d: &Rlp<'_>) -> Result<Self, DecoderError> {
        match d.as_val().expect("rlp decode Error") {
            LOCAL => Ok(TxOrigin::Local),
            EXTERNAL => Ok(TxOrigin::External),
            _ => Err(DecoderError::Custom("Unexpected Txorigin type")),
        }
    }
}

impl PartialOrd for TxOrigin {
    fn partial_cmp(&self, other: &TxOrigin) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TxOrigin {
    fn cmp(&self, other: &TxOrigin) -> Ordering {
        if *other == *self {
            return Ordering::Equal
        }

        match (*self, *other) {
            (TxOrigin::Local, _) => Ordering::Less,
            _ => Ordering::Greater,
        }
    }
}

impl TxOrigin {
    pub fn is_local(self) -> bool {
        self == TxOrigin::Local
    }
}

#[derive(Clone, Copy, Debug)]
/// Light structure used to identify transaction and its order
pub struct TransactionOrder {
    /// Memory usage of this transaction.
    /// Currently using the RLP byte length of the transaction as the mem usage.
    pub mem_usage: usize,
    /// Hash to identify associated transaction
    pub hash: TxHash,
    /// Incremental id assigned when transaction is inserted to the pool.
    pub insertion_id: u64,
    /// Origin of the transaction
    pub origin: TxOrigin,
}

impl TransactionOrder {
    pub fn for_transaction(item: &MemPoolItem) -> Self {
        let rlp_bytes_len = rlp::encode(&item.tx).len();
        ctrace!(MEM_POOL, "New tx with size {}", rlp_bytes_len);
        Self {
            mem_usage: rlp_bytes_len,
            hash: item.hash(),
            insertion_id: item.insertion_id,
            origin: item.origin,
        }
    }
}

impl Eq for TransactionOrder {}
impl PartialEq for TransactionOrder {
    fn eq(&self, other: &TransactionOrder) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}
impl PartialOrd for TransactionOrder {
    fn partial_cmp(&self, other: &TransactionOrder) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for TransactionOrder {
    fn cmp(&self, b: &TransactionOrder) -> Ordering {
        // Local transactions should always have priority
        if self.origin != b.origin {
            return self.origin.cmp(&b.origin)
        }

        // Lastly compare insertion_id
        self.insertion_id.cmp(&b.insertion_id)
    }
}

/// Transaction item in the mem pool.
#[derive(Clone, Eq, PartialEq, Debug, RlpEncodable)]
pub struct MemPoolItem {
    /// Transaction.
    pub tx: VerifiedTransaction,
    /// Transaction origin.
    pub origin: TxOrigin,
    /// Insertion time
    pub inserted_block_number: PoolingInstant,
    /// Insertion timstamp
    pub inserted_timestamp: u64,
    /// ID assigned upon insertion, should be unique.
    pub insertion_id: u64,
}

impl MemPoolItem {
    pub fn new(
        tx: VerifiedTransaction,
        origin: TxOrigin,
        inserted_block_number: PoolingInstant,
        inserted_timestamp: u64,
        insertion_id: u64,
    ) -> Self {
        MemPoolItem {
            tx,
            origin,
            inserted_block_number,
            inserted_timestamp,
            insertion_id,
        }
    }

    pub fn hash(&self) -> TxHash {
        self.tx.hash()
    }
}

#[derive(Debug, PartialEq)]
pub struct CurrentQueue {
    /// Priority queue for transactions
    pub queue: BTreeSet<TransactionOrder>,
    /// Memory usage of the external transactions in the queue
    pub mem_usage: usize,
    /// Count of the external transactions in the queue
    pub count: usize,
}

impl CurrentQueue {
    pub fn new() -> Self {
        Self {
            queue: BTreeSet::new(),
            mem_usage: 0,
            count: 0,
        }
    }

    pub fn clear(&mut self) {
        self.queue.clear();
        self.mem_usage = 0;
        self.count = 0;
    }

    pub fn len(&self) -> usize {
        self.queue.len()
    }

    pub fn insert(&mut self, order: TransactionOrder) {
        self.queue.insert(order);
        if !order.origin.is_local() {
            self.mem_usage += order.mem_usage;
            self.count += 1;
        }
    }

    pub fn remove(&mut self, order: &TransactionOrder) {
        assert!(self.queue.remove(order));
        if !order.origin.is_local() {
            self.mem_usage -= order.mem_usage;
            self.count -= 1;
        }
    }
}
