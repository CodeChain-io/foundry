// Copyright 2018-2020 Kodebox, Inc.
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

use crate::consensus::{ConsensusEngine, Evidence, TendermintSealView};
use crate::error::{BlockError, Error};
use ccrypto::BLAKE_NULL_RLP;
use ckey::Ed25519Public as Public;
use coordinator::engine::{BlockExecutor, ExecutionId};
use coordinator::types::Event;
use coordinator::{Header as PreHeader, Transaction, TransactionWithMetadata};
use cstate::{CurrentValidatorSet, NextValidatorSet, StateDB, StateError, StateWithCache, TopLevelState, TopState};
use ctypes::header::{Header, Seal};
use ctypes::util::unexpected::Mismatch;
use ctypes::{CompactValidatorSet, ConsensusParams, TxHash};
use merkle_trie::skewed_merkle_root;
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::collections::{HashMap, HashSet};

/// A block, encoded as it is on the block chain.
#[derive(Debug, Clone)]
pub struct Block {
    /// The header of this block
    pub header: Header,
    /// The evidences in this block
    pub evidences: Vec<Evidence>,
    /// The transactions in this block.
    pub transactions: Vec<Transaction>,
}

impl Block {
    /// Get the RLP-encoding of the block with or without the seal.
    pub fn rlp_bytes(&self, seal: &Seal) -> Bytes {
        let mut block_rlp = RlpStream::new_list(3);
        self.header.stream_rlp(&mut block_rlp, seal);
        block_rlp.append_list(&self.evidences);
        block_rlp.append_list(&self.transactions);
        block_rlp.out()
    }
}

impl Decodable for Block {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let got = rlp.as_raw().len();
        let expected = rlp.payload_info()?.total();
        if got > expected {
            return Err(DecoderError::RlpIsTooBig {
                expected,
                got,
            })
        }
        if got < expected {
            return Err(DecoderError::RlpIsTooShort {
                expected,
                got,
            })
        }
        let item_count = rlp.item_count()?;
        if rlp.item_count()? != 3 {
            return Err(DecoderError::RlpIncorrectListLen {
                expected: 3,
                got: item_count,
            })
        }
        Ok(Block {
            header: rlp.val_at(0)?,
            evidences: rlp.list_at(1)?,
            transactions: rlp.list_at(2)?,
        })
    }
}

/// An internal type for a block's common elements.
#[derive(Clone)]
pub struct ExecutedBlock {
    header: Header,
    state: TopLevelState,
    evidences: Vec<Evidence>,
    transactions: Vec<Transaction>,
    tx_events: HashMap<TxHash, Vec<Event>>,
    block_events: Vec<Event>,
    transactions_set: HashSet<TxHash>,
}

impl ExecutedBlock {
    fn new(state: TopLevelState, parent: &Header) -> ExecutedBlock {
        ExecutedBlock {
            header: parent.generate_child(),
            state,
            evidences: Default::default(),
            transactions: Default::default(),
            tx_events: Default::default(),
            block_events: Default::default(),
            transactions_set: Default::default(),
        }
    }
    /// Get mutable access to a state.
    pub fn state_mut(&mut self) -> &mut TopLevelState {
        &mut self.state
    }

    pub fn transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

/// Block that is ready for transactions to be added.
pub struct OpenBlock {
    execution_id: Option<ExecutionId>,
    block: ExecutedBlock,
}

impl OpenBlock {
    /// Create a new `OpenBlock` ready for transaction pushing.
    pub fn try_new(
        engine: &dyn ConsensusEngine,
        db: StateDB,
        parent: &Header,
        author: Public,
        evidences: Vec<Evidence>,
        extra_data: Bytes,
    ) -> Result<Self, Error> {
        let state = TopLevelState::from_existing(db, *parent.state_root()).map_err(StateError::from)?;
        let mut block = ExecutedBlock::new(state, parent);
        let header = &mut block.header;

        header.set_author(author);
        header.set_extra_data(extra_data);
        header.note_dirty();
        header.set_evidences_root(skewed_merkle_root(BLAKE_NULL_RLP, evidences.iter().map(Encodable::rlp_bytes)));

        block.evidences = evidences;

        engine.populate_from_parent(header, parent);

        Ok(OpenBlock {
            execution_id: None,
            block,
        })
    }

    pub fn open(&mut self, block_executor: &dyn BlockExecutor, engine: &dyn ConsensusEngine) -> Result<(), Error> {
        let last_committed_validators = {
            let validator_bitset = TendermintSealView::new(self.header().seal())
                .bitset()
                .map_err(|_| Error::Block(BlockError::InvalidSeal))?;
            let possible_authors = engine.possible_authors(None)?.expect("Tendermint must have possible authors");
            let committed_validators_result: Result<Vec<_>, _> = validator_bitset
                .true_index_iter()
                .map(|index| possible_authors.get(index).map(Clone::clone).ok_or(Error::Block(BlockError::InvalidSeal)))
                .collect();
            committed_validators_result?
        };

        let pre_header = PreHeader::new(
            *self.header().parent_hash(),
            self.header().timestamp(),
            self.header().number(),
            *self.header().author(),
            last_committed_validators,
            self.header().extra_data().clone(),
        );
        let verified_crimes: Vec<_> = self.block.evidences.iter().map(|e| e.into()).collect();
        self.execution_id = Some(block_executor.open_block(self.block.state_mut(), &pre_header, &verified_crimes)?);

        Ok(())
    }

    pub fn execute_transactions(
        &mut self,
        block_executor: &dyn BlockExecutor,
        mut transactions: Vec<Transaction>,
    ) -> Result<(), Error> {
        let execution_id = self.execution_id.expect("Txs can be executed only after opening a block");
        // TODO: Handle erroneous transactions
        let transaction_results = block_executor
            .execute_transactions(execution_id, self.inner_mut().state_mut(), &transactions)
            .map_err(|_| Error::Other(String::from("Rejected while executing transactions")))?;
        self.block.transactions.append(&mut transactions);
        // TODO: How to do this without copy?
        let mut tx_events: HashMap<TxHash, Vec<Event>> = HashMap::new();
        for (tx, result) in transactions.into_iter().zip(transaction_results.into_iter()) {
            tx_events.insert(tx.hash(), result.events);
        }
        self.block.tx_events = tx_events;
        Ok(())
    }

    /// Populate self from a header.
    fn populate_from(&mut self, header: &Header) {
        self.block.header.set_timestamp(header.timestamp());
        self.block.header.set_author(*header.author());
        self.block.header.set_extra_data(header.extra_data().clone());
        self.block.header.set_seal(header.seal().to_vec());
    }

    pub fn prepare_block_from_transactions<'a>(
        &mut self,
        block_executor: &dyn BlockExecutor,
        mut transactions: impl Iterator<Item = &'a TransactionWithMetadata> + 'a,
    ) {
        let execution_id = self.execution_id.expect("A block can be prepared only after opening the block");
        let proposed_txs = block_executor.prepare_block(execution_id, self.block.state_mut(), &mut transactions);
        self.block.transactions.append(&mut proposed_txs.iter().map(|(tx, _)| (*tx).clone()).collect());
        self.block.tx_events = proposed_txs.into_iter().map(|(tx, outcome)| (tx.hash(), outcome.events)).collect();
    }

    /// Turn this into a `ClosedBlock`.
    pub fn close(mut self, block_executor: &dyn BlockExecutor) -> Result<ClosedBlock, Error> {
        let execution_id = self.execution_id.expect("A block can be closed only when it's open");
        let block_outcome = block_executor.close_block(execution_id)?;

        self.block.block_events = block_outcome.events;
        let updated_validator_set = block_outcome.updated_validator_set;
        let next_validator_set_hash = match updated_validator_set {
            Some(ref set) => set.hash(),
            None => NextValidatorSet::load_from_state(self.block.state())?.create_compact_validator_set().hash(),
        };
        let updated_consensus_params = block_outcome.updated_consensus_params;
        if let Err(e) = self.update_next_block_state(updated_validator_set, updated_consensus_params) {
            warn!("Encountered error on closing the block: {}", e);
            return Err(e)
        }

        let state_root = self.block.state_mut().commit().map_err(|e| {
            warn!("Encountered error on state commit: {}", e);
            e
        })?;
        self.block.header.set_state_root(state_root);

        self.block.header.set_next_validator_set_hash(next_validator_set_hash);

        if self.block.header.transactions_root() == &BLAKE_NULL_RLP {
            self.block.header.set_transactions_root(skewed_merkle_root(
                BLAKE_NULL_RLP,
                self.block.transactions.iter().map(Encodable::rlp_bytes),
            ));
        }
        debug_assert_eq!(
            self.block.header.transactions_root(),
            &skewed_merkle_root(BLAKE_NULL_RLP, self.block.transactions.iter().map(Encodable::rlp_bytes),)
        );

        // FIXME: update tx events and block event

        Ok(ClosedBlock {
            block: self.block,
        })
    }

    /// Alter the timestamp of the block.
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.block.header.set_timestamp(timestamp);
    }

    /// Provide a valid seal
    ///
    /// NOTE: This does not check the validity of `seal` with the engine.
    pub fn seal(&mut self, engine: &dyn ConsensusEngine, seal: Vec<Bytes>) -> Result<(), BlockError> {
        let expected_seal_fields = engine.seal_fields(self.header());
        if seal.len() != expected_seal_fields {
            return Err(BlockError::InvalidSealArity(Mismatch {
                expected: expected_seal_fields,
                found: seal.len(),
            }))
        }
        self.block.header.set_seal(seal);
        Ok(())
    }

    pub fn inner_mut(&mut self) -> &mut ExecutedBlock {
        &mut self.block
    }

    // called on open_block
    fn update_current_validator_set(&mut self) -> Result<(), Error> {
        let mut current_validators = CurrentValidatorSet::load_from_state(&self.state())?;
        current_validators.update(NextValidatorSet::load_from_state(&self.state())?);
        current_validators.save_to_state(self.inner_mut().state_mut())?;

        Ok(())
    }

    // called on close_block
    fn update_next_block_state(
        &mut self,
        updated_validator_set: Option<CompactValidatorSet>,
        updated_consensus_params: Option<ConsensusParams>,
    ) -> Result<(), Error> {
        let state = self.block.state_mut();

        if let Some(set) = updated_validator_set {
            let validators = NextValidatorSet::from_compact_validator_set(set);
            validators.save_to_state(state)?;
        }

        if let Some(params) = updated_consensus_params {
            state.update_consensus_params(params)?;
        }
        Ok(())
    }
}

#[derive(Clone)]
pub struct ClosedBlock {
    block: ExecutedBlock,
}

impl ClosedBlock {
    /// Get the hash of the header without seal arguments.
    pub fn hash(&self) -> H256 {
        self.header().rlp_blake(&Seal::Without)
    }

    pub fn rlp_bytes(&self) -> Bytes {
        let mut block_rlp = RlpStream::new_list(3);
        self.block.header.stream_rlp(&mut block_rlp, &Seal::With);
        block_rlp.append_list(&self.block.evidences);
        block_rlp.append_list(&self.block.transactions);
        block_rlp.out()
    }
}

pub trait IsBlock {
    /// Get the `ExecutedBlock` associated with this object.
    fn block(&self) -> &ExecutedBlock;

    /// Get the base `Block` object associated with this.
    fn to_base(&self) -> Block {
        Block {
            header: self.header().clone(),
            evidences: self.evidences().to_vec(),
            transactions: self.transactions().to_vec(),
        }
    }

    /// Get the header associated with this object's block.
    fn header(&self) -> &Header {
        &self.block().header
    }

    /// Get all information on transactions in this block.
    fn transactions(&self) -> &[Transaction] {
        &self.block().transactions
    }

    /// Get all information on evidences in this block.
    fn evidences(&self) -> &[Evidence] {
        &self.block().evidences
    }

    /// Get exclusive access to the final state associated with this object's block.
    fn state(&self) -> &TopLevelState {
        &self.block().state
    }

    /// Get the events of each transaction in this block
    fn tx_events(&self) -> &HashMap<TxHash, Vec<Event>> {
        &self.block().tx_events
    }

    /// Get the events emitted by this block
    fn block_events(&self) -> &Vec<Event> {
        &self.block().block_events
    }
}

impl IsBlock for ExecutedBlock {
    fn block(&self) -> &ExecutedBlock {
        self
    }
}

impl IsBlock for OpenBlock {
    fn block(&self) -> &ExecutedBlock {
        &self.block
    }
}

impl IsBlock for ClosedBlock {
    fn block(&self) -> &ExecutedBlock {
        &self.block
    }
}

/// Enact the block given by block header, transactions and uncles
pub fn enact(
    header: &Header,
    evidences: Vec<Evidence>,
    transactions: &[Transaction],
    engine: &dyn ConsensusEngine,
    block_executor: &dyn BlockExecutor,
    db: StateDB,
    parent: &Header,
) -> Result<ClosedBlock, Error> {
    let mut b = OpenBlock::try_new(engine, db, parent, Public::default(), evidences, vec![])?;

    b.populate_from(header);
    b.update_current_validator_set()?;

    b.open(block_executor, engine)?;
    b.execute_transactions(block_executor, transactions.to_vec())?;
    b.close(block_executor)
}
