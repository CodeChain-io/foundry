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

use super::super::{ConsensusEngine, EngineError, Seal};
use super::network::TendermintExtension;
pub use super::params::{TendermintParams, TimeoutParams};
use super::worker;
use super::Evidence;
use super::{ChainNotify, Tendermint, SEAL_FIELDS};
use crate::account_provider::AccountProvider;
use crate::block::*;
use crate::client::snapshot_notify::NotifySender as SnapshotNotifySender;
use crate::client::{Client, ConsensusClient};
use crate::consensus::tendermint::params::TimeGapParams;
use crate::consensus::EngineType;
use crate::error::Error;
use crate::views::HeaderView;
use crate::BlockId;
use ckey::Address;
use cnetwork::NetworkService;
use crossbeam_channel as crossbeam;
use cstate::{CurrentValidatorSet, NextValidatorSet, TopState};
use ctypes::{BlockHash, CompactValidatorSet, ConsensusParams, Header};
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
    fn on_open_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
        let mut current_validators = CurrentValidatorSet::load_from_state(block.state())?;
        current_validators.update(NextValidatorSet::load_from_state(block.state())?);
        current_validators.save_to_state(block.state_mut())?;

        Ok(())
    }

    fn on_close_block(
        &self,
        block: &mut ExecutedBlock,
        updated_validator_set: CompactValidatorSet,
        updated_consensus_params: ConsensusParams,
    ) -> Result<(), Error> {
        let state = block.state_mut();

        let validators = NextValidatorSet::from_compact_validator_set(updated_validator_set);
        validators.save_to_state(state)?;

        state.update_consensus_params(updated_consensus_params)?;
        Ok(())
    }

    fn register_client(&self, client: Weak<dyn ConsensusClient>) {
        *self.client.write() = Some(Weak::clone(&client));
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

    fn set_signer(&self, ap: Arc<AccountProvider>, address: Address) {
        self.has_signer.store(true, AtomicOrdering::SeqCst);
        self.inner
            .send(worker::Event::SetSigner {
                ap,
                address,
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

    fn possible_authors(&self, block_number: Option<u64>) -> Result<Option<Vec<Address>>, EngineError> {
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
        Ok(Some(self.validators.next_addresses(&block_hash)))
    }
}
