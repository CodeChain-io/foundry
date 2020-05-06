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

mod params;

use self::params::SoloParams;
use super::stake;
use super::{ConsensusEngine, Seal};
use crate::block::ExecutedBlock;
use crate::client::snapshot_notify::NotifySender;
use crate::client::ConsensusClient;
use crate::consensus::{EngineError, EngineType};
use crate::error::Error;
use ckey::Ed25519Public as Public;
use cstate::{init_stake, DoubleVoteHandler, StateDB, StateResult, StateWithCache, TopLevelState};
use ctypes::transaction::Action;
use ctypes::{BlockHash, Header};
use parking_lot::RwLock;
use primitives::H256;
use std::collections::HashMap;
use std::sync::{Arc, Weak};

/// A consensus engine which does not provide any consensus mechanism.
pub struct Solo {
    client: RwLock<Option<Weak<dyn ConsensusClient>>>,
    snapshot_notify_sender: Arc<RwLock<Option<NotifySender>>>,
    genesis_stakes: HashMap<Public, u64>,
    stake: stake::Stake,
}

impl Solo {
    /// Returns new instance of Solo over the given state machine.
    pub fn new(params: SoloParams) -> Self {
        let genesis_stakes = params.genesis_stakes;

        Solo {
            client: Default::default(),
            snapshot_notify_sender: Arc::new(RwLock::new(None)),
            genesis_stakes,
            stake: stake::Stake::default(),
        }
    }
}

impl ConsensusEngine for Solo {
    fn seals_internally(&self) -> bool {
        true
    }

    fn engine_type(&self) -> EngineType {
        EngineType::Solo
    }

    fn generate_seal(&self, _block: Option<&ExecutedBlock>, _parent: &Header) -> Seal {
        Seal::Solo
    }

    fn close_block_actions(&self, _block: &ExecutedBlock) -> Result<Vec<Action>, Error> {
        Ok(vec![])
    }

    fn register_client(&self, client: Weak<dyn ConsensusClient>) {
        *self.client.write() = Some(Weak::clone(&client));
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

    fn stake_handler(&self) -> Option<&dyn DoubleVoteHandler> {
        Some(&self.stake)
    }

    fn possible_authors(&self, _block_number: Option<u64>) -> Result<Option<Vec<Public>>, EngineError> {
        Ok(None)
    }

    fn initialize_genesis_state(&self, db: StateDB, root: H256) -> StateResult<(StateDB, H256)> {
        let mut top_level = TopLevelState::from_existing(db, root)?;
        init_stake(&mut top_level, self.genesis_stakes.clone(), Default::default(), Default::default())?;
        Ok(top_level.commit_and_into_db()?)
    }

    fn current_validator_set(
        &self,
        _block_number: Option<u64>,
    ) -> Result<Option<ctypes::CompactValidatorSet>, EngineError> {
        Ok(Some(ctypes::CompactValidatorSet::new(Vec::new())))
    }
}

#[cfg(test)]
mod tests {
    use crate::scheme::Scheme;
    use ctypes::Header;

    #[test]
    fn fail_to_verify() {
        let engine = Scheme::new_test_solo().engine;
        let header: Header = Header::default();

        assert!(engine.verify_header_basic(&header).is_ok());
    }
}
