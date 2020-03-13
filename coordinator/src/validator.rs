use ckey::Ed25519Public as Public;
use ctypes::{BlockHash, TxHash};

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

pub struct ValidatorInfo {
    weight: VoteWeight,
    pubkey: Public,
}

pub struct ConsensusParams {
    /// Validators' public keys with their voting powers.
    pub validators: Vec<ValidatorInfo>,
    // Note: This code is copied from json/src/tendermint.rs
    /// Propose step timeout in milliseconds.
    pub timeout_propose: Option<u64>,
    /// Propose step timeout delta in milliseconds.
    pub timeout_propose_delta: Option<u64>,
    /// Prevote step timeout in milliseconds.
    pub timeout_prevote: Option<u64>,
    /// Prevote step timeout delta in milliseconds.
    pub timeout_prevote_delta: Option<u64>,
    /// Precommit step timeout in milliseconds.
    pub timeout_precommit: Option<u64>,
    /// Precommit step timeout delta in milliseconds.
    pub timeout_precommit_delta: Option<u64>,
    /// Commit step timeout in milliseconds.
    pub timeout_commit: Option<u64>,
    /// Reward per block.
    pub block_reward: Option<u64>,
    /// allowed past time gap in milliseconds.
    pub allowed_past_timegap: Option<u64>,
    /// allowed future time gap in milliseconds.
    pub allowed_future_timegap: Option<u64>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: u64,
    /// Block author.
    author: Public,
    /// Block extra data.
    extra_data: Bytes,
}

/// A decoded transaction.
pub struct Transaction {
    tx_type: String,
    body: Bytes,
}

impl Transaction {
    fn tx_type(&self) -> &str {
        &self.tx_type
    }

    fn body<T: 'static>(&self) -> Option<&T> {
        unimplemented!()
    }

    pub fn size(&self) -> usize {
        unimplemented!()
    }

    pub fn hash(&self) -> TxHash {
        unimplemented!()
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
    pub is_success: bool,
    pub events: Vec<Event>,
}

pub struct BlockOutcome {
    pub block_hash: BlockHash,
    pub updated_consensus_params: ConsensusParams,
    pub transaction_results: Vec<TransactionExecutionOutcome>,
    pub events: Vec<Event>,
}

pub trait Validator {
    fn initialize_chain(&mut self) -> ConsensusParams;
    fn execute_block(&mut self, header: &Header, transactions: &[Transaction], evidences: &[Evidence]) -> BlockOutcome;
    fn check_transaction(&mut self, transaction: &Transaction) -> bool;
    fn fetch_transactions_for_block(&self, transactions: Vec<&TransactionWithMetadata>) -> Vec<&TransactionWithGas>;
    fn remove_transactions(
        &mut self,
        transactions: &[TransactionWithMetadata],
        size: Option<usize>,
    ) -> (Vec<&TransactionWithMetadata>, Vec<&TransactionWithMetadata>);
}
