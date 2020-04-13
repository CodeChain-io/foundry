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

use super::core::Orderer;
use super::get_seq;
use super::types::{OrdererError, TxMetadata};
use ckey::Address;
use coordinator::validator::TxOrigin;
use ctypes::TxHash;
use std::cmp::Ordering;
use std::convert::TryFrom;

#[allow(unused)]
// SeqFeeOrderer will filter and order transactions using seqence and fee metadata
pub struct SeqFeeOrderer {}

fn into_address(_: &str) -> Address {
    unimplemented!()
}

fn into_u64(_: &str) -> u64 {
    unimplemented!()
}

fn get_sender_from_metadata(metadata: &TxMetadata) -> Result<Address, OrdererError> {
    metadata
        .custom_metadata
        .get("sender")
        .ok_or(OrdererError::MetadataFieldNotFound {
            field: "sender",
            tx_hash: metadata.tx_hash,
        })
        .map(|sender| into_address(sender))
}

fn get_seq_from_metadata(metadata: &TxMetadata) -> Result<u64, OrdererError> {
    metadata
        .custom_metadata
        .get("seq")
        .ok_or(OrdererError::MetadataFieldNotFound {
            field: "seq",
            tx_hash: metadata.tx_hash,
        })
        .map(|seq| into_u64(seq))
}

fn get_fee_from_metadata(metadata: &TxMetadata) -> Result<u64, OrdererError> {
    metadata
        .custom_metadata
        .get("fee")
        .ok_or(OrdererError::MetadataFieldNotFound {
            field: "fee",
            tx_hash: metadata.tx_hash,
        })
        .map(|fee| into_u64(fee))
}

#[derive(Debug)]
pub struct TransactionOrder {
    /// Primary ordering factory. Difference between transaction seq and expected seq in state
    /// (e.g. Transaction(seq:5), State(seq:0) -> height: 5)
    /// High seq_height = Low priority (processed later)
    pub seq_height: u64,
    /// Fee of the transaction.
    pub fee: u64,
    /// Fee per bytes(rlp serialized) of the transaction
    pub fee_per_byte: u64,
    /// Memory usage of this transaction.
    /// Incremental id assigned when transaction is inserted to the pool.
    pub insertion_id: u64,
    /// Origin of the transaction
    pub origin: TxOrigin,
}

#[derive(Debug)]
pub enum InvalidTransactionError {
    Old(TxHash),
}

#[derive(Debug)]
pub enum ConversionError {
    CannotConvert(OrdererError),
    InvalidTransaction(InvalidTransactionError),
}

impl From<InvalidTransactionError> for ConversionError {
    fn from(e: InvalidTransactionError) -> Self {
        ConversionError::InvalidTransaction(e)
    }
}

impl From<OrdererError> for ConversionError {
    fn from(e: OrdererError) -> Self {
        ConversionError::CannotConvert(e)
    }
}

impl TryFrom<TxMetadata> for TransactionOrder {
    type Error = ConversionError;
    fn try_from(metadata: TxMetadata) -> Result<Self, Self::Error> {
        let sender = get_sender_from_metadata(&metadata)?;
        let sender_seq = get_seq(&sender);

        let tx_seq = get_seq_from_metadata(&metadata)?;

        if tx_seq <= sender_seq {
            return Err(InvalidTransactionError::Old(metadata.tx_hash).into())
        }

        let fee = get_fee_from_metadata(&metadata)?;
        // TODO: Maybe we can check fee <= get_balance(sender) here

        let tx_size = metadata.mem_usage;

        let fee_per_byte = fee / tx_size as u64;

        Ok(Self {
            seq_height: sender_seq - tx_seq,
            fee,
            fee_per_byte,
            insertion_id: metadata.insertion_id,
            origin: metadata.origin,
        })
    }
}

fn compare_origin(a: TxOrigin, b: TxOrigin) -> Ordering {
    if a == b {
        return Ordering::Equal
    }

    match (a, b) {
        (TxOrigin::Local, _) => Ordering::Less,
        _ => Ordering::Greater,
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
            return compare_origin(self.origin, b.origin)
        }

        // Check seq_height
        if self.seq_height != b.seq_height {
            return self.seq_height.cmp(&b.seq_height)
        }

        if self.fee_per_byte != b.fee_per_byte {
            return self.fee_per_byte.cmp(&b.fee_per_byte)
        }

        // Then compare fee
        if self.fee != b.fee {
            return b.fee.cmp(&self.fee)
        }

        // Lastly compare insertion_id
        self.insertion_id.cmp(&b.insertion_id)
    }
}

impl Orderer for SeqFeeOrderer {
    fn filter_and_order_transactions(&self, metadata_list: Vec<TxMetadata>) -> Result<Vec<usize>, OrdererError> {
        let (oks, fails): (Vec<_>, Vec<_>) =
            metadata_list.into_iter().map(TransactionOrder::try_from).partition(Result::is_ok);

        let mut ords: Vec<(usize, TransactionOrder)> = oks.into_iter().map(Result::unwrap).enumerate().collect();
        let errors: Vec<ConversionError> = fails.into_iter().map(Result::unwrap_err).collect();

        for e in errors {
            // TODO: How to log error in module?
            println!("{:?}", e);
            match e {
                ConversionError::CannotConvert(e) => return Err(e),
                ConversionError::InvalidTransaction(_) => (),
            }
        }

        ords.sort_by(|(_, ord1), (_, ord2)| ord1.cmp(ord2));

        Ok(ords.into_iter().map(|(idx, _)| idx).collect())
    }
}
