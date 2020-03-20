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

mod params;

use self::params::SoloParams;
use super::stake;
use super::{ConsensusEngine, Seal};
use crate::block::ExecutedBlock;
use crate::client::snapshot_notify::NotifySender;
use crate::client::ConsensusClient;
use crate::codechain_machine::CodeChainMachine;
use crate::consensus::{EngineError, EngineType};
use crate::error::Error;
use ckey::Address;
use cstate::ActionHandler;
use ctypes::{BlockHash, Header};
use parking_lot::RwLock;
use std::sync::{Arc, Weak};

/// A consensus engine which does not provide any consensus mechanism.
pub struct Solo {
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
    machine: CodeChainMachine,
    action_handlers: Vec<Arc<dyn ActionHandler>>,
    snapshot_notify_sender: Arc<RwLock<Option<NotifySender>>>,
}

impl Solo {
    /// Returns new instance of Solo over the given state machine.
    pub fn new(params: SoloParams, machine: CodeChainMachine) -> Self {
        let mut action_handlers: Vec<Arc<dyn ActionHandler>> = Vec::new();
        action_handlers.push(Arc::new(stake::Stake::new(
            params.genesis_stakes,
            Default::default(),
            Default::default(),
        )));

        Solo {
            client: Default::default(),
            machine,
            action_handlers,
            snapshot_notify_sender: Arc::new(RwLock::new(None)),
        }
    }

    fn client(&self) -> Option<Arc<dyn ConsensusClient>> {
        self.client.read().as_ref()?.upgrade()
    }
}

impl ConsensusEngine for Solo {
    fn name(&self) -> &str {
        "Solo"
    }

    fn machine(&self) -> &CodeChainMachine {
        &self.machine
    }

    fn seals_internally(&self) -> bool {
        true
    }

    fn engine_type(&self) -> EngineType {
        EngineType::Solo
    }

    fn generate_seal(&self, _block: Option<&ExecutedBlock>, _parent: &Header) -> Seal {
        Seal::Solo
    }

    fn on_close_block(&self, block: &mut ExecutedBlock) -> Result<(), Error> {
        let client = self.client().ok_or(EngineError::CannotOpenBlock)?;

        let parent_hash = *block.header().parent_hash();
        let parent = client.block_header(&parent_hash.into()).expect("Parent header must exist");
        let parent_common_params = client.common_params(parent_hash.into()).expect("CommonParams of parent must exist");
        let term_seconds = parent_common_params.term_seconds();
        if term_seconds == 0 {
            return Ok(())
        }
        let last_term_finished_block_num = {
            let header = block.header();
            let current_term_period = header.timestamp() / term_seconds;
            let parent_term_period = parent.timestamp() / term_seconds;
            if current_term_period == parent_term_period {
                return Ok(())
            }
            header.number()
        };

        stake::on_term_close(block.state_mut(), last_term_finished_block_num, &[])?;
        Ok(())
    }

    fn register_client(&self, client: Weak<dyn ConsensusClient>) {
        *self.client.write() = Some(Weak::clone(&client));
    }

    fn recommended_confirmation(&self) -> u32 {
        1
    }

    fn register_snapshot_notify_sender(&self, sender: NotifySender) {
        let mut guard = self.snapshot_notify_sender.write();
        assert!(guard.is_none(), "snapshot_notify_sender is registered twice");
        *guard = Some(sender);
    }

    fn send_snapshot_notify(&self, block_hash: BlockHash) {
        if let Some(sender) = self.snapshot_notify_sender.read().as_ref() {
            sender.notify(block_hash)
        }
    }

    fn action_handlers(&self) -> &[Arc<dyn ActionHandler>] {
        &self.action_handlers
    }

    fn possible_authors(&self, _block_number: Option<u64>) -> Result<Option<Vec<Address>>, EngineError> {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use crate::scheme::Scheme;
    use ctypes::Header;
    use primitives::H520;

    #[test]
    fn fail_to_verify() {
        let engine = Scheme::new_test_solo().engine;
        let mut header: Header = Header::default();

        assert!(engine.verify_header_basic(&header).is_ok());

        header.set_seal(vec![::rlp::encode(&H520::default())]);

        assert!(engine.verify_block_seal(&header).is_ok());
    }
}
