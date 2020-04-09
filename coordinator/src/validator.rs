use super::context::SubStorageAccess;
use ccrypto::blake256;
use ckey::Address;
use ctypes::{CompactValidatorSet, TxHash};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

/// A `Validator` receives requests from the underlying consensus engine
/// and performs validation of blocks and Txes.
///
///

pub type Bytes = Vec<u8>;

pub type VoteWeight = u64;

pub struct Event {
    pub key: &'static str,
    pub value: Bytes,
}

pub struct ConsensusParams {
    /// Validators' public keys with their voting powers.
    pub validators: CompactValidatorSet,
    // Note: This code is copied from json/src/tendermint.rs
    /// Propose step timeout in milliseconds.
    pub timeout_propose: u64,
    /// Propose step timeout delta in milliseconds.
    pub timeout_propose_delta: u64,
    /// Prevote step timeout in milliseconds.
    pub timeout_prevote: u64,
    /// Prevote step timeout delta in milliseconds.
    pub timeout_prevote_delta: u64,
    /// Precommit step timeout in milliseconds.
    pub timeout_precommit: u64,
    /// Precommit step timeout delta in milliseconds.
    pub timeout_precommit_delta: u64,
    /// Commit step timeout in milliseconds.
    pub timeout_commit: u64,
    /// Reward per block.
    pub block_reward: u64,
    /// allowed past time gap in milliseconds.
    pub allowed_past_timegap: u64,
    /// allowed future time gap in milliseconds.
    pub allowed_future_timegap: u64,
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

pub struct TransactionWithGas {
    pub tx: Transaction,
    pub gas: usize,
}

impl<'a> TransactionWithGas {
    fn new(tx: Transaction, gas: usize) -> Self {
        Self {
            tx,
            gas,
        }
    }

    pub fn size(&self) -> usize {
        self.tx.size()
    }

    pub fn hash(&self) -> TxHash {
        self.tx.hash()
    }
}
pub enum Evidence {
    DoubleVote, // Should import and use DoubleVote type defined in tendermint module?
}

pub struct TransactionExecutionOutcome {
    pub events: Vec<Event>,
}

pub struct BlockOutcome {
    pub is_success: bool,
    pub updated_consensus_params: ConsensusParams,
    pub transaction_results: Vec<TransactionExecutionOutcome>,
    pub events: Vec<Event>,
}

pub type ErrorCode = i64;

pub trait Validator {
    fn initialize_chain(&self) -> ConsensusParams;
    fn open_block(&self, context: &mut dyn SubStorageAccess, header: &Header, evidences: &[Evidence]);
    fn execute_transactions(&self, context: &mut dyn SubStorageAccess, transactions: &[Transaction]);
    fn close_block(&self, context: &mut dyn SubStorageAccess) -> BlockOutcome;
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;
    fn fetch_transactions_for_block<'a>(
        &self,
        transactions: &'a [&'a TransactionWithMetadata],
    ) -> Vec<&'a TransactionWithGas>;
    fn remove_transactions<'a>(
        &self,
        transactions: &'a [&'a TransactionWithMetadata],
        memory_limit: Option<usize>,
        size_limit: Option<usize>,
    ) -> (Vec<&'a TransactionWithMetadata>, Vec<&'a TransactionWithMetadata>);
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn encode_and_decode_transaction() {
        let transaction = Transaction::new("test".to_string(), vec![0, 1, 2, 3, 4]);
        rlp_encode_and_decode_test!(transaction);
    }
}
