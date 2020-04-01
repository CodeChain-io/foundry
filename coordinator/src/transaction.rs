// Copyright 2020 Kodebox, Inc.
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

use ccrypto::blake256;
use ctypes::TxHash;
use primitives::Bytes;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;

/// An encoded transaction.
#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct Transaction {
    tx_type: String,
    body: Bytes,
}

impl Transaction {
    pub fn new(tx_type: String, body: Bytes) -> Self {
        Self {
            tx_type,
            body,
        }
    }

    pub fn tx_type(&self) -> &str {
        &self.tx_type
    }

    pub fn body(&self) -> &Bytes {
        &self.body
    }

    pub fn size(&self) -> usize {
        self.rlp_bytes().len()
    }

    pub fn hash(&self) -> TxHash {
        blake256(self.rlp_bytes()).into()
    }
}

impl Encodable for Transaction {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2).append(&self.tx_type).append(self.body());
    }
}

impl Decodable for Transaction {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 2 {
            return Err(DecoderError::RlpIncorrectListLen {
                expected: 2,
                got: item_count,
            })
        }
        Ok(Self {
            tx_type: rlp.val_at(0)?,
            body: rlp.val_at(1)?,
        })
    }
}

/// Transaction origin
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn is_external(self) -> bool {
        self == TxOrigin::External
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct TransactionWithMetadata {
    pub tx: Transaction,
    pub origin: TxOrigin,
    pub inserted_block_number: u64,
    pub inserted_timestamp: u64,
    /// ID assigned upon insertion, should be unique
    pub insertion_id: u64,
}

impl<'a> TransactionWithMetadata {
    pub fn new(
        tx: Transaction,
        origin: TxOrigin,
        inserted_block_number: u64,
        inserted_timestamp: u64,
        insertion_id: u64,
    ) -> Self {
        Self {
            tx,
            origin,
            inserted_block_number,
            inserted_timestamp,
            insertion_id,
        }
    }

    pub fn size(&self) -> usize {
        self.tx.size()
    }

    pub fn hash(&self) -> TxHash {
        self.tx.hash()
    }
}

impl Encodable for TransactionWithMetadata {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(5)
            .append(&self.tx)
            .append(&self.origin)
            .append(&self.inserted_block_number)
            .append(&self.inserted_timestamp)
            .append(&self.insertion_id);
    }
}

impl Decodable for TransactionWithMetadata {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 5 {
            return Err(DecoderError::RlpIncorrectListLen {
                expected: 5,
                got: item_count,
            })
        }
        Ok(Self {
            tx: rlp.val_at(0)?,
            origin: rlp.val_at(1)?,
            inserted_block_number: rlp.val_at(2)?,
            inserted_timestamp: rlp.val_at(3)?,
            insertion_id: rlp.val_at(4)?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn encode_and_decode_transaction() {
        let transaction = Transaction {
            tx_type: "test".to_string(),
            body: vec![0, 1, 2, 3, 4],
        };
        rlp_encode_and_decode_test!(transaction);
    }
}
