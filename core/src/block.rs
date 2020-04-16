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

use super::invoice::Invoice;
use crate::client::{EngineInfo, TermInfo};
use crate::consensus::CodeChainEngine;
use crate::error::{BlockError, Error};
use crate::transaction::{UnverifiedTransaction, VerifiedTransaction};
use ccrypto::BLAKE_NULL_RLP;
use ckey::Address;
use cstate::{FindDoubleVoteHandler, NextValidators, StateDB, StateError, StateWithCache, TopLevelState};
use ctypes::errors::HistoryError;
use ctypes::header::{Header, Seal};
use ctypes::util::unexpected::Mismatch;
use ctypes::{BlockNumber, TxHash};
use merkle_trie::skewed_merkle_root;
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::collections::HashSet;

/// A block, encoded as it is on the block chain.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// The header of this block
    pub header: Header,
    /// The transactions in this block.
    pub transactions: Vec<UnverifiedTransaction>,
}

impl Block {
    /// Get the RLP-encoding of the block with or without the seal.
    pub fn rlp_bytes(&self, seal: &Seal) -> Bytes {
        let mut block_rlp = RlpStream::new_list(2);
        self.header.stream_rlp(&mut block_rlp, seal);
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
        if rlp.item_count()? != 2 {
            return Err(DecoderError::RlpIncorrectListLen {
                expected: 2,
                got: item_count,
            })
        }
        Ok(Block {
            header: rlp.val_at(0)?,
            transactions: rlp.list_at(1)?,
        })
    }
}

/// An internal type for a block's common elements.
#[derive(Clone)]
pub struct ExecutedBlock {
    header: Header,
    state: TopLevelState,
    transactions: Vec<VerifiedTransaction>,
    invoices: Vec<Invoice>,
    transactions_set: HashSet<TxHash>,
}

impl ExecutedBlock {
    fn new(state: TopLevelState, parent: &Header) -> ExecutedBlock {
        ExecutedBlock {
            header: parent.generate_child(),
            state,
            transactions: Default::default(),
            invoices: Default::default(),
            transactions_set: Default::default(),
        }
    }

    /// Get mutable access to a state.
    pub fn state_mut(&mut self) -> &mut TopLevelState {
        &mut self.state
    }

    pub fn transactions(&self) -> &[VerifiedTransaction] {
        &self.transactions
    }

    pub fn header(&self) -> &Header {
        &self.header
    }
}

/// Block that is ready for transactions to be added.
pub struct OpenBlock<'x> {
    block: ExecutedBlock,
    _engine: &'x dyn CodeChainEngine,
}

impl<'x> OpenBlock<'x> {
    /// Create a new `OpenBlock` ready for transaction pushing.
    pub fn try_new(
        engine: &'x dyn CodeChainEngine,
        db: StateDB,
        parent: &Header,
        author: Address,
        extra_data: Bytes,
    ) -> Result<Self, Error> {
        let state = TopLevelState::from_existing(db, *parent.state_root()).map_err(StateError::from)?;
        let mut r = OpenBlock {
            block: ExecutedBlock::new(state, parent),
            _engine: engine,
        };

        r.block.header.set_author(author);
        r.block.header.set_extra_data(extra_data);
        r.block.header.note_dirty();

        engine.populate_from_parent(&mut r.block.header, parent);

        Ok(r)
    }

    /// Push a transaction into the block.
    pub fn push_transaction<C: FindDoubleVoteHandler>(
        &mut self,
        tx: VerifiedTransaction,
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
    ) -> Result<(), Error> {
        if self.block.transactions_set.contains(&tx.hash()) {
            return Err(HistoryError::TransactionAlreadyImported.into())
        }

        let hash = tx.hash();
        let error = match self.block.state.apply(
            &tx.transaction(),
            &tx.signer_public(),
            client,
            parent_block_number,
            parent_block_timestamp,
            self.block.header.timestamp(),
        ) {
            Ok(()) => {
                self.block.transactions_set.insert(hash);
                self.block.transactions.push(tx);
                None
            }
            Err(err) => Some(err),
        };
        self.block.invoices.push(Invoice {
            hash,
            error: error.clone().map(|err| err.to_string()),
        });

        match error {
            None => Ok(()),
            Some(err) => Err(err.into()),
        }
    }

    /// Push transactions onto the block.
    pub fn push_transactions<C: FindDoubleVoteHandler>(
        &mut self,
        transactions: &[VerifiedTransaction],
        client: &C,
        parent_block_number: BlockNumber,
        parent_block_timestamp: u64,
    ) -> Result<(), Error> {
        for tx in transactions {
            self.push_transaction(tx.clone(), client, parent_block_number, parent_block_timestamp)?;
        }
        Ok(())
    }

    /// Populate self from a header.
    fn populate_from(&mut self, header: &Header) {
        self.block.header.set_timestamp(header.timestamp());
        self.block.header.set_author(*header.author());
        self.block.header.set_extra_data(header.extra_data().clone());
        self.block.header.set_seal(header.seal().to_vec());
    }

    /// Turn this into a `ClosedBlock`.
    pub fn close(mut self) -> Result<ClosedBlock, Error> {
        let state_root = self.block.state.commit().map_err(|e| {
            warn!("Encountered error on state commit: {}", e);
            e
        })?;
        self.block.header.set_state_root(state_root);

        let vset_raw = NextValidators::load_from_state(self.block.state())?;
        let vset = vset_raw.create_compact_validator_set();
        self.block.header.set_next_validator_set_hash(vset.hash());

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

    /// Alter the timestamp of the block.
    pub fn set_timestamp(&mut self, timestamp: u64) {
        self.block.header.set_timestamp(timestamp);
    }

    /// Provide a valid seal
    ///
    /// NOTE: This does not check the validity of `seal` with the engine.
    pub fn seal(&mut self, engine: &dyn CodeChainEngine, seal: Vec<Bytes>) -> Result<(), BlockError> {
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
            transactions: self.transactions().iter().cloned().map(Into::into).collect(),
        }
    }

    /// Get the header associated with this object's block.
    fn header(&self) -> &Header {
        &self.block().header
    }

    /// Get all information on transactions in this block.
    fn transactions(&self) -> &[VerifiedTransaction] {
        &self.block().transactions
    }

    /// Get all information on receipts in this block.
    fn invoices(&self) -> &[Invoice] {
        &self.block().invoices
    }

    /// Get the final state associated with this object's block.
    fn state(&self) -> &TopLevelState {
        &self.block().state
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
pub fn enact<C: EngineInfo + FindDoubleVoteHandler + TermInfo>(
    header: &Header,
    transactions: &[VerifiedTransaction],
    engine: &dyn CodeChainEngine,
    client: &C,
    db: StateDB,
    parent: &Header,
) -> Result<ClosedBlock, Error> {
    let mut b = OpenBlock::try_new(engine, db, parent, Address::default(), vec![])?;

    b.populate_from(header);
    b.push_transactions(transactions, client, parent.number(), parent.timestamp())?;

    b.close()
}
