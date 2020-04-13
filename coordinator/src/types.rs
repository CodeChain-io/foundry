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

use ccrypto::blake256;
use ckey::Address;
use ctypes::{CompactValidatorSet, ConsensusParams, TxHash};
use primitives::Bytes;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Event {
    pub key: String,
    pub value: Bytes,
}

impl Encodable for Event {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(2).append(&self.key).append(&self.value);
    }
}

impl Decodable for Event {
    fn decode(rlp: &Rlp) -> Result<Self, rlp::DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count != 2 {
            return Err(DecoderError::RlpIncorrectListLen {
                expected: 2,
                got: item_count,
            })
        }
        Ok(Self {
            key: rlp.val_at(0)?,
            value: rlp.val_at(1)?,
        })
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: u64,
    /// Block author.
    author: Address,
    /// Block extra data.
    extra_data: Bytes,
}

impl Header {
    pub fn new(timestamp: u64, number: u64, author: Address, extra_data: Bytes) -> Self {
        Self {
            timestamp,
            number,
            author,
            extra_data,
        }
    }
}

/// A decoded transaction.
#[derive(Clone, Debug, Eq, PartialEq)]
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

#[derive(Clone, Debug, Eq, PartialEq)]
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

// TransactionWithGas will be returned by fetch_transactions_for_block
pub struct TransactionWithGas<'a> {
    pub tx_with_metadata: &'a TransactionWithMetadata,
    pub gas: usize,
}

impl<'a> TransactionWithGas<'a> {
    fn new(tx_with_metadata: &'a TransactionWithMetadata, gas: usize) -> Self {
        Self {
            tx_with_metadata,
            gas,
        }
    }

    pub fn size(&self) -> usize {
        self.tx_with_metadata.size()
    }

    pub fn hash(&self) -> TxHash {
        self.tx_with_metadata.hash()
    }
}
pub enum VerifiedCrime {
    DoubleVote {
        height: u64,
        author_index: usize,
        criminal_index: usize,
    },
}

pub struct TransactionExecutionOutcome {
    pub events: Vec<Event>,
}

pub struct BlockOutcome {
    pub is_success: bool,
    pub updated_validator_set: CompactValidatorSet,
    pub updated_consensus_params: ConsensusParams,
    pub transaction_results: Vec<TransactionExecutionOutcome>,
    pub events: Vec<Event>,
}

// Error code returned by check_transaction
pub type ErrorCode = i64;

#[cfg(test)]
mod tests {
    use super::*;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn encode_and_decode_events() {
        let event = Event {
            key: "test key".to_string(),
            value: vec![0, 1, 2, 3, 4, 5],
        };
        rlp_encode_and_decode_test!(event);
    }

    #[test]
    fn encode_and_decode_transaction() {
        let transaction = Transaction::new("test".to_string(), vec![0, 1, 2, 3, 4]);
        rlp_encode_and_decode_test!(transaction);
    }
}
