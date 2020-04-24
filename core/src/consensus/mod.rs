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

mod bit_set;
mod null_engine;
pub(crate) mod signer;
mod solo;
pub mod stake;
mod tendermint;
mod validator_set;

pub use self::null_engine::NullEngine;
pub use self::solo::Solo;
pub use self::tendermint::{
    ConsensusMessage, Height, Step, Tendermint, TendermintParams, TimeGapParams, View, VoteOn, VoteStep,
};
pub use self::validator_set::{DynamicValidator, ValidatorSet};

use self::bit_set::BitSet;
use crate::account_provider::AccountProvider;
use crate::block::{ClosedBlock, ExecutedBlock};
use crate::client::snapshot_notify::NotifySender as SnapshotNotifySender;
use crate::client::ConsensusClient;
use crate::error::Error;
use crate::transaction::UnverifiedTransaction;
use crate::views::HeaderView;
use crate::Client;
use ckey::{Address, Signature};
use cnetwork::NetworkService;
use cstate::{DoubleVoteHandler, StateDB, StateResult};
use ctypes::errors::SyntaxError;
use ctypes::transaction::Action;
use ctypes::util::unexpected::{Mismatch, OutOfBounds};
use ctypes::{BlockHash, CommonParams, Header};
use primitives::{Bytes, H256};
use std::fmt;
use std::sync::{Arc, Weak};

pub enum Seal {
    Solo,
    Tendermint {
        prev_view: View,
        cur_view: View,
        precommits: Vec<Signature>,
        precommit_bitset: BitSet,
    },
    None,
}

impl Seal {
    pub fn seal_fields(&self) -> Option<Vec<Bytes>> {
        match self {
            Seal::None => None,
            Seal::Solo => Some(Vec::new()),
            Seal::Tendermint {
                prev_view,
                cur_view,
                precommits,
                precommit_bitset,
            } => Some(vec![
                ::rlp::encode(prev_view),
                ::rlp::encode(cur_view),
                ::rlp::encode_list(precommits),
                ::rlp::encode(precommit_bitset),
            ]),
        }
    }
}

/// Engine type.
#[derive(Debug, PartialEq, Eq)]
pub enum EngineType {
    PBFT,
    Solo,
}

impl EngineType {
    pub fn need_signer_key(&self) -> bool {
        match self {
            EngineType::PBFT => true,
            EngineType::Solo => false,
        }
    }

    pub fn ignore_reseal_min_period(&self) -> bool {
        match self {
            EngineType::PBFT => true,
            EngineType::Solo => false,
        }
    }

    pub fn ignore_reseal_on_transaction(&self) -> bool {
        match self {
            EngineType::PBFT => true,
            EngineType::Solo => false,
        }
    }
}

/// A consensus mechanism for the chain.
pub trait ConsensusEngine: Sync + Send {
    /// The number of additional header fields required for this engine.
    fn seal_fields(&self, _header: &Header) -> usize {
        0
    }

    /// true means the engine is currently prime for seal generation (i.e. node is the current validator).
    /// false means that the node might seal internally but is not qualified now.
    fn seals_internally(&self) -> bool;

    /// The type of this engine.
    fn engine_type(&self) -> EngineType;

    /// Attempt to seal the block internally.
    ///
    /// If `Some` is returned, then you get a valid seal.
    ///
    /// This operation is synchronous and may (quite reasonably) not be available, in which None will
    /// be returned.
    ///
    /// It is fine to require access to state or a full client for this function, since
    /// light clients do not generate seals.
    fn generate_seal(&self, _block: Option<&ExecutedBlock>, _parent: &Header) -> Seal {
        Seal::None
    }

    fn proposal_generated(&self, _block: &ClosedBlock) {}

    /// Phase 1 quick block verification. Only does checks that are cheap. Returns either a null `Ok` or a general error detailing the problem with import.
    fn verify_header_basic(&self, _header: &Header) -> Result<(), Error> {
        Ok(())
    }

    /// Phase 3 verification. Check block information against parent. Returns either a null `Ok` or a general error detailing the problem with import.
    /// The verification must be conducted only with the two headers' information because it does not guarantee whether the two corresponding bodies have been imported.
    fn verify_block_family(&self, _header: &Header, _parent: &Header) -> Result<(), Error> {
        Ok(())
    }

    /// Phase 4 verification. Verify block header against potentially external data.
    /// Should only be called when `register_client` has been called previously.
    fn verify_block_external(&self, _header: &Header) -> Result<(), Error> {
        Ok(())
    }

    /// Populate a header's fields based on its parent's header.
    /// Usually implements the chain scoring rule based on weight.
    fn populate_from_parent(&self, _header: &mut Header, _parent: &Header) {}

    /// Called when the step is not changed in time
    fn on_timeout(&self, _token: usize) {}

    /// Block transformation functions, before the transactions.
    fn open_block_action(&self, _block: &ExecutedBlock) -> Result<Option<Action>, Error> {
        Ok(None)
    }

    /// Block transformation functions, after the transactions.
    fn close_block_actions(&self, _block: &ExecutedBlock) -> Result<Vec<Action>, Error> {
        Ok(vec![])
    }

    /// Add Client which can be used for sealing, potentially querying the state and sending messages.
    fn register_client(&self, _client: Weak<dyn ConsensusClient>) {}

    /// Find out if the block is a proposal block and should not be inserted into the DB.
    /// Takes a header of a fully verified block.
    fn is_proposal(&self, _verified_header: &Header) -> bool {
        false
    }

    /// Register an account which signs consensus messages.
    fn set_signer(&self, _ap: Arc<AccountProvider>, _address: Address) {}

    fn register_network_extension_to_service(&self, _: &NetworkService) {}

    fn register_time_gap_config_to_worker(&self, _time_gap_params: TimeGapParams) {}

    fn register_chain_notify(&self, _: &Client) {}

    fn complete_register(&self) {}

    fn register_snapshot_notify_sender(&self, _sender: SnapshotNotifySender) {}

    fn send_snapshot_notify(&self, _block_hash: BlockHash) {}

    fn get_best_block_from_best_proposal_header(&self, header: &HeaderView<'_>) -> BlockHash {
        header.hash()
    }

    /// In Tendermint consensus, the highest scored block may not be the best block.
    /// Only the descendant of the current best block could be the next best block in Tendermint consensus.
    fn can_change_canon_chain(
        &self,
        _parent_hash_of_new_header: BlockHash,
        _grandparent_hash_of_new_header: BlockHash,
        _previous_best_hash: BlockHash,
    ) -> bool {
        true
    }

    fn stake_handler(&self) -> Option<&dyn DoubleVoteHandler> {
        None
    }

    fn possible_authors(&self, block_number: Option<u64>) -> Result<Option<Vec<Address>>, EngineError>;

    fn initialize_genesis_state(&self, db: StateDB, root: H256) -> StateResult<(StateDB, H256)> {
        Ok((db, root))
    }
}

/// Voting errors.
#[derive(Debug)]
pub enum EngineError {
    /// Precommit signatures or author field does not belong to an authority.
    BlockNotAuthorized(Address),
    /// The signature cannot be verified with the signer of the message.
    MessageWithInvalidSignature {
        height: u64,
        signer_index: usize,
        address: Address,
    },
    /// The vote for the future height couldn't be verified
    FutureMessage {
        future_height: u64,
        current_height: u64,
    },
    /// The validator on the given height and index is exist(index >= validator set size)
    ValidatorNotExist {
        height: u64,
        index: usize,
    },
    /// The same author issued different votes at the same step.
    DoubleVote(Address),
    /// The received block is from an incorrect proposer.
    NotProposer(Mismatch<Address>),
    /// Seal field has an unexpected size.
    BadSealFieldSize(OutOfBounds<usize>),
    /// Malformed consensus message.
    MalformedMessage(String),
    CannotOpenBlock,
}

impl fmt::Display for EngineError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use self::EngineError::*;
        let msg = match self {
            BlockNotAuthorized(address) => format!("Signer {} is not authorized.", address),
            MessageWithInvalidSignature {
                height,
                signer_index,
                address,
            } => format!("The {}th validator({}) on height {} is not authorized.", signer_index, address, height),
            FutureMessage {
                future_height,
                current_height,
            } => format!("The message is from height {} but the current height is {}", future_height, current_height),
            ValidatorNotExist {
                height,
                index,
            } => format!("The {}th validator on height {} does not exist. (out of bound)", index, height),
            DoubleVote(address) => format!("Author {} issued too many blocks.", address),
            NotProposer(mis) => format!("Author is not a current proposer: {}", mis),
            BadSealFieldSize(oob) => format!("Seal field has an unexpected length: {}", oob),
            MalformedMessage(msg) => format!("Received malformed consensus message: {}", msg),
            CannotOpenBlock => "Cannot open a block".to_string(),
        };

        f.write_fmt(format_args!("Engine error ({})", msg))
    }
}

/// Common type alias for an engine coupled with an CodeChain-like state machine.
pub trait CodeChainEngine: ConsensusEngine {
    /// Additional verification for transactions in blocks.
    fn verify_transaction_with_params(
        &self,
        tx: &UnverifiedTransaction,
        common_params: &CommonParams,
    ) -> Result<(), Error> {
        if let Action::ReportDoubleVote {
            message1,
            message2,
        } = &tx.transaction().action
        {
            let handler =
                self.stake_handler().ok_or_else(|| SyntaxError::InvalidCustomAction("no valid handler".to_string()))?;
            handler.verify(message1, message2)?;
        }
        tx.verify_with_params(common_params)?;
        Ok(())
    }
}

// convenience wrappers for existing functions.
impl<T> CodeChainEngine for T where T: ConsensusEngine {}
