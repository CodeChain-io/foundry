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

use super::super::{ConsensusEngine, EngineError, Seal};
use super::network::TendermintExtension;
pub use super::params::{TendermintParams, TimeoutParams};
use super::{worker, Evidence};
use super::{ChainNotify, Step, Tendermint, VoteOn, VoteStep, SEAL_FIELDS};
use crate::account_provider::AccountProvider;
use crate::block::*;
use crate::client::snapshot_notify::NotifySender as SnapshotNotifySender;
use crate::client::{Client, ConsensusClient};
use crate::consensus::tendermint::params::TimeGapParams;
use crate::consensus::{EngineType, TendermintSealView};
use crate::error::{BlockError, Error};
use crate::views::HeaderView;
use ckey::{verify, Ed25519Public as Public};
use cnetwork::NetworkService;
use crossbeam_channel as crossbeam;
use cstate::{CurrentValidators, DoubleVoteHandler, Jail, NextValidators, TopStateView};
use ctypes::transaction::Action;
use ctypes::{util::unexpected::OutOfBounds, BlockHash, BlockId, CompactValidatorSet, Header, SyncHeader};
use std::iter::Iterator;
use std::sync::atomic::Ordering as AtomicOrdering;
use std::sync::{Arc, Weak};

impl ConsensusEngine for Tendermint {
    /// (consensus view, proposal signature, authority signatures)
    fn seal_fields(&self, _header: &Header) -> usize {
        SEAL_FIELDS
    }

    /// Should this node participate.
    fn seals_internally(&self) -> bool {
        self.has_signer.load(AtomicOrdering::SeqCst)
    }

    fn engine_type(&self) -> EngineType {
        EngineType::PBFT
    }

    /// Attempt to seal generate a proposal seal.
    ///
    /// This operation is synchronous and may (quite reasonably) not be available, in which case
    /// `Seal::None` will be returned.
    fn generate_seal(&self, _block: Option<&ExecutedBlock>, parent: &Header) -> Seal {
        let (result, receiver) = crossbeam::bounded(1);
        let parent_hash = parent.hash();
        self.inner
            .send(worker::Event::GenerateSeal {
                block_number: parent.number() + 1,
                parent_hash,
                result,
            })
            .unwrap();
        receiver.recv().unwrap()
    }

    /// Called when the node is the leader and a proposal block is generated from the miner.
    /// This writes the proposal information and go to the prevote step.
    fn proposal_generated(&self, block: &ClosedBlock) {
        self.inner.send(worker::Event::ProposalGenerated(Box::from(block.clone()))).unwrap();
    }

    fn verify_header_basic(&self, header: &Header) -> Result<(), Error> {
        let (result, receiver) = crossbeam::bounded(1);
        self.inner
            .send(worker::Event::VerifyHeaderBasic {
                header: Box::from(header.clone()),
                result,
            })
            .unwrap();
        receiver.recv().unwrap()
    }

    /// This function is very similar to the verify_block_external in the tendermint/worker.rs file
    fn verify_header_seal(&self, header: &Header, validator_set: &CompactValidatorSet) -> Result<(), Error> {
        if header.number() <= 1 {
            return Ok(())
        }
        let seal_view = TendermintSealView::new(header.seal());
        let bitset_count = seal_view.bitset()?.count();
        let precommits_count = seal_view.precommits().item_count()?;

        if bitset_count < precommits_count {
            cwarn!(
                ENGINE,
                "verify_header_seal: The header({})'s bitset count is less than the precommits count",
                header.hash()
            );
            return Err(BlockError::InvalidSeal.into())
        }

        if bitset_count > precommits_count {
            cwarn!(
                ENGINE,
                "verify_header_seal: The header({})'s bitset count is greater than the precommits count",
                header.hash()
            );
            return Err(BlockError::InvalidSeal.into())
        }

        let parent_block_finalized_view = seal_view.parent_block_finalized_view()?;
        let precommit_vote_on = VoteOn {
            step: VoteStep::new(header.number() - 1, parent_block_finalized_view, Step::Precommit),
            block_hash: Some(*header.parent_hash()),
        };

        let mut signed_delegation: u64 = 0;
        for (bitset_index, signature) in seal_view.signatures()? {
            if validator_set.len() <= bitset_index {
                cwarn!(
                    ENGINE,
                    "verify_header_seal: The header({})'s bitset index({}) is greater than or equal to the validator set length({})",
                    header.hash(),
                    bitset_index,
                    validator_set.len()
                );
                return Err(BlockError::InvalidSeal.into())
            }
            let public = validator_set[bitset_index].public_key;
            let delegation = validator_set[bitset_index].delegation;
            if !verify(&signature, &precommit_vote_on.hash(), &public) {
                return Err(EngineError::BlockNotAuthorized(public).into())
            }
            signed_delegation += delegation;
        }

        let total_delegation: u64 = validator_set.iter().map(|entry| entry.delegation).sum();

        if signed_delegation * 3 > total_delegation * 2 {
            Ok(())
        } else {
            Err(EngineError::BadSealFieldSize(OutOfBounds {
                min: Some(total_delegation as usize * 2 / 3),
                max: Some(total_delegation as usize),
                found: signed_delegation as usize,
            })
            .into())
        }
    }

    fn verify_block_external(&self, header: &Header) -> Result<(), Error> {
        let (result, receiver) = crossbeam::bounded(1);
        self.inner
            .send(worker::Event::VerifyBlockExternal {
                header: Box::from(header.clone()),
                result,
            })
            .unwrap();
        receiver.recv().unwrap()
    }

    /// Equivalent to a timeout: to be used for tests.
    fn on_timeout(&self, token: usize) {
        self.inner.send(worker::Event::OnTimeout(token)).unwrap();
    }

    /// Block transformation functions, before the transactions.
    fn open_block_action(&self, block: &ExecutedBlock) -> Result<Option<Action>, Error> {
        Ok(Some(Action::UpdateValidators {
            validators: NextValidators::load_from_state(block.state())?.into(),
        }))
    }

    fn close_block_actions(&self, block: &ExecutedBlock) -> Result<Vec<Action>, Error> {
        let client = self.client().ok_or(EngineError::CannotOpenBlock)?;

        let parent_hash = *block.header().parent_hash();
        let parent = client.block_header(&parent_hash.into()).expect("Parent header must exist").decode();
        let parent_consensus_params =
            client.consensus_params(parent_hash.into()).expect("ConsensusParams of parent must exist");

        let metadata = block.state().metadata()?.expect("Metadata must exist");

        let term = metadata.current_term_id();
        let term_seconds = match term {
            0 => parent_consensus_params.term_seconds(),
            _ => {
                let parent_term_common_params = client.term_common_params(parent_hash.into());
                parent_term_common_params.expect("TermCommonParams should exist").term_seconds()
            }
        };
        let next_validators = NextValidators::update_weight(block.state(), block.header().author())?;
        if !is_term_changed(block.header(), &parent, term_seconds) {
            return Ok(vec![Action::ChangeNextValidators {
                validators: next_validators.into(),
            }])
        }

        let current_term = metadata.current_term_id();
        let (custody_until, kick_at) = {
            let params = metadata.params();
            let custody_period = params.custody_period();
            assert_ne!(0, custody_period);
            let release_period = params.release_period();
            assert_ne!(0, release_period);
            (current_term + custody_period, current_term + release_period)
        };

        let released_addresses = Jail::load_from_state(block.state())?.released_addresses(current_term);

        Ok(vec![
            Action::CloseTerm {
                inactive_validators: vec![],
                next_validators: next_validators.into(),
                released_addresses,
                custody_until,
                kick_at,
            },
            Action::Elect {},
        ])
    }

    fn register_client(&self, client: Weak<dyn ConsensusClient>) {
        *self.client.write() = Some(Weak::clone(&client));
        self.stake.register_resources(client, Arc::downgrade(&self.validators));
    }

    fn fetch_evidences(&self) -> Vec<Evidence> {
        let (result, receiver) = crossbeam::bounded(1);
        self.inner
            .send(worker::Event::FetchEvidences {
                result,
            })
            .unwrap();
        receiver.recv().unwrap()
    }

    fn remove_published_evidences(&self, published: Vec<Evidence>) {
        self.inner
            .send(worker::Event::RemovePublishedEvidences {
                published,
            })
            .unwrap();
    }

    fn is_proposal(&self, header: &Header) -> bool {
        let (result, receiver) = crossbeam::bounded(1);
        self.inner
            .send(worker::Event::IsProposal {
                block_number: header.number(),
                block_hash: header.hash(),
                result,
            })
            .unwrap();
        receiver.recv().unwrap()
    }

    fn set_signer(&self, ap: Arc<AccountProvider>, pubkey: Public) {
        self.has_signer.store(true, AtomicOrdering::SeqCst);
        self.inner
            .send(worker::Event::SetSigner {
                ap,
                pubkey,
            })
            .unwrap();
    }

    fn register_network_extension_to_service(&self, service: &NetworkService) {
        let timeouts = self.timeouts;

        let inner = self.inner.clone();
        let extension = service.register_extension(move |api| TendermintExtension::new(inner, timeouts, api));
        let client = Arc::downgrade(&self.client().unwrap());
        self.extension_initializer.send((extension, client)).unwrap();
    }

    fn register_time_gap_config_to_worker(&self, time_gap_params: TimeGapParams) {
        self.external_params_initializer.send(time_gap_params).unwrap();
    }

    fn register_chain_notify(&self, client: &Client) {
        client.add_notify(Arc::downgrade(&self.chain_notify) as Weak<dyn ChainNotify>);
    }

    fn complete_register(&self) {
        let (result, receiver) = crossbeam::bounded(1);
        self.inner.send(worker::Event::Restore(result)).unwrap();
        receiver.recv().unwrap();
    }

    fn register_snapshot_notify_sender(&self, sender: SnapshotNotifySender) {
        self.snapshot_notify_sender_initializer.send(sender).unwrap();
    }

    fn get_best_block_from_best_proposal_header(&self, header: &HeaderView<'_>) -> BlockHash {
        header.parent_hash()
    }

    fn can_change_canon_chain(
        &self,
        parent_hash_of_new_header: BlockHash,
        grandparent_hash_of_new_header: BlockHash,
        prev_best_hash: BlockHash,
    ) -> bool {
        parent_hash_of_new_header == prev_best_hash || grandparent_hash_of_new_header == prev_best_hash
    }

    fn stake_handler(&self) -> Option<&dyn DoubleVoteHandler> {
        Some(&*self.stake)
    }

    fn possible_authors(&self, block_number: Option<u64>) -> Result<Option<Vec<Public>>, EngineError> {
        let client = self.client().ok_or(EngineError::CannotOpenBlock)?;
        let header = match block_number {
            None => {
                client.block_header(&BlockId::Latest).expect("latest block must exist")
                // the latest block
            }
            Some(block_number) => {
                assert_ne!(0, block_number);
                client.block_header(&(block_number - 1).into()).ok_or(EngineError::CannotOpenBlock)?
                // the parent of the given block number
            }
        };
        let block_hash = header.hash();
        Ok(Some(self.validators.next_validators(&block_hash)))
    }

    fn current_validator_set(&self, block_number: Option<u64>) -> Result<Option<CompactValidatorSet>, EngineError> {
        let client = self.client().ok_or(EngineError::CannotOpenBlock)?;
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        let state = match client.state_at(block_id) {
            None => return Ok(None),
            Some(state) => state,
        };
        if block_number != Some(0) {
            Ok(Some(
                CurrentValidators::load_from_state(&state)
                    .expect("We read state from verified block")
                    .create_compact_validator_set(),
            ))
        } else {
            Ok(None)
        }
    }

    /// grand_parent === none only when parent is genesis
    fn verify_header_family(
        &self,
        header: &SyncHeader,
        parent: &Header,
        grand_parent: Option<&Header>,
    ) -> Result<(), Error> {
        let grand_parent = match grand_parent {
            Some(header) => header,
            None => {
                debug_assert_eq!(parent.number(), 0);
                return Ok(())
            }
        };

        // next validator hash of grand parent is not correct
        if grand_parent.number() == 0 {
            return Ok(())
        }

        let parent_validator_hash = grand_parent.next_validator_set_hash();
        let parent_validator = header.prev_validator_set().expect("Currently all child should have validator_set");
        if parent_validator_hash != &parent_validator.hash() {
            cwarn!(SYNC, "Received headers have invalid validator set:\n  grand parent: (height: {}, hash: {}, next_validator_set_hash: {}),\n  child: (height: {}, hash: {}, hash(validator_set): {})", grand_parent.number(), grand_parent.hash(), parent_validator_hash,
                               header.number(), header.hash(), parent_validator.hash());
            return Err(BlockError::InvalidValidatorSet.into())
        }

        Ok(())
    }
}

pub(super) fn is_term_changed(header: &Header, parent: &Header, term_seconds: u64) -> bool {
    // Because the genesis block has a fixed generation time, the first block should not change the term.
    if header.number() == 1 {
        return false
    }
    if term_seconds == 0 {
        return false
    }

    let current_term_period = header.timestamp() / term_seconds;
    let parent_term_period = parent.timestamp() / term_seconds;

    current_term_period != parent_term_period
}
