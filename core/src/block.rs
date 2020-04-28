// Copyright 2018-2020 Kodebox, Inc.
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

use crate::consensus::{ConsensusEngine, Evidence};
use crate::error::{BlockError, Error};
use ccrypto::BLAKE_NULL_RLP;
use ckey::Address;
use coordinator::traits::BlockExecutor;
use coordinator::types::{Event, Header as PreHeader, Transaction, VerifiedCrime};
use cstate::{NextValidatorSet, StateDB, StateError, StateWithCache, TopLevelState};
use ctypes::header::{Header, Seal};
use ctypes::util::unexpected::Mismatch;
use ctypes::TxHash;
use merkle_trie::skewed_merkle_root;
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::collections::HashMap;

/// A block, encoded as it is on the block chain.
#[derive(Debug, Clone, PartialEq)]
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
pub struct OpenBlock<'x> {
    block: ExecutedBlock,
    engine: &'x dyn ConsensusEngine,
}

impl<'x> OpenBlock<'x> {
    /// Create a new `OpenBlock` ready for transaction pushing.
    pub fn try_new(
        engine: &'x dyn ConsensusEngine,
        db: StateDB,
        parent: &Header,
        author: Address,
        extra_data: Bytes,
    ) -> Result<Self, Error> {
        let state = TopLevelState::from_existing(db, *parent.state_root()).map_err(StateError::from)?;
        let mut r = OpenBlock {
            block: ExecutedBlock::new(state, parent),
            engine,
        };

        r.block.header.set_author(author);
        r.block.header.set_extra_data(extra_data);
        r.block.header.note_dirty();

        engine.populate_from_parent(&mut r.block.header, parent);

        Ok(r)
    }

    pub fn open(&mut self, block_executor: &dyn BlockExecutor, evidences: Vec<Evidence>) -> Result<(), Error> {
        let pre_header = PreHeader::new(
            self.header().timestamp(),
            self.header().number(),
            *self.header().author(),
            self.header().extra_data().clone(),
        );
        let verified_crimes: Vec<VerifiedCrime> = evidences.iter().map(|e| e.into()).collect();
        self.block.evidences = evidences;

        if self.block.header.evidences_root() == &BLAKE_NULL_RLP {
            self.block.header.set_evidences_root(skewed_merkle_root(
                BLAKE_NULL_RLP,
                self.block.evidences.iter().map(Encodable::rlp_bytes),
            ));
        }
        block_executor.open_block(self.state_mut(), &pre_header, &verified_crimes).map_err(From::from)
    }

    pub fn execute_transactions(
        &mut self,
        block_executor: &dyn BlockExecutor,
        mut transactions: Vec<Transaction>,
    ) -> Result<(), Error> {
        // TODO: Handle erroneous transactions
        let transaction_results = block_executor
            .execute_transactions(self.state_mut(), &transactions)
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

    /// Turn this into a `ClosedBlock`.
    pub fn close(mut self, block_executor: &dyn BlockExecutor) -> Result<ClosedBlock, Error> {
        let block_outcome = block_executor.close_block(self.state_mut())?;

        self.block.block_events = block_outcome.events;
        let updated_validator_set = block_outcome.updated_validator_set;
        let next_validator_set_hash = match updated_validator_set {
            Some(ref set) => set.hash(),
            None => NextValidatorSet::load_from_state(self.block.state())?.create_compact_validator_set().hash(),
        };
        let updated_consensus_params = block_outcome.updated_consensus_params;
        if let Err(e) = self.engine.on_close_block(&mut self.block, updated_validator_set, updated_consensus_params) {
            warn!("Encountered error on closing the block: {}", e);
            return Err(e)
        }

        let state_root = self.block.state.commit().map_err(|e| {
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

        Ok(ClosedBlock {
            block: self.block,
        })
    }

    /// Populate self from a header.
    fn populate_from(&mut self, header: &Header) {
        self.block.header.set_timestamp(header.timestamp());
        self.block.header.set_author(*header.author());
        self.block.header.set_extra_data(header.extra_data().clone());
        self.block.header.set_seal(header.seal().to_vec());
    }

    /// Alter the timestamp of the block.
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.block.header.set_timestamp(timestamp);
    }

    /// Provide a valid seal
    ///
    /// NOTE: This does not check the validity of `seal` with the engine.
    pub fn seal(&mut self, seal: Vec<Bytes>) -> Result<(), BlockError> {
        let expected_seal_fields = self.engine.seal_fields(self.header());
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

    fn state_mut(&mut self) -> &mut TopLevelState {
        &mut self.block.state
    }
}

/// Just like `OpenBlock`, except that we've applied `Engine::on_close_block`, finished up the non-seal header fields.
///
/// There is no function available to push a transaction.
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
        let mut block_rlp = RlpStream::new_list(2);
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

    /// Get all information on evidences in this block.
    fn evidences(&self) -> &[Evidence] {
        &self.block().evidences
    }

    /// Get all information on transactions in this block.
    fn transactions(&self) -> &[Transaction] {
        &self.block().transactions
    }

    /// Get the final state associated with this object's block.
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

impl<'x> IsBlock for OpenBlock<'x> {
    fn block(&self) -> &ExecutedBlock {
        &self.block
    }
}

impl<'x> IsBlock for ClosedBlock {
    fn block(&self) -> &ExecutedBlock {
        &self.block
    }
}

/// Enact the block given by block header, transactions and uncles
pub fn enact(
    header: &Header,
    transactions: &[Transaction],
    evidences: &[Evidence],
    engine: &dyn ConsensusEngine,
    block_executor: &dyn BlockExecutor,
    db: StateDB,
    parent: &Header,
) -> Result<ClosedBlock, Error> {
    let mut b = OpenBlock::try_new(engine, db, parent, Address::default(), vec![])?;

    b.populate_from(header);
    engine.on_open_block(b.inner_mut())?;

    b.open(block_executor, evidences.to_vec())?;
    b.execute_transactions(block_executor, transactions.to_vec())?;
    b.close(block_executor)
}
