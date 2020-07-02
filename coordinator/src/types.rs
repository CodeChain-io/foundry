// Copyright 2020 Kodebox, Inc.
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

mod event;

pub use self::event::Event;
use ctypes::{CompactValidatorSet, ConsensusParams};

pub enum VerifiedCrime {
    #[allow(dead_code)]
    DoubleVote {
        height: u64,
        author_index: usize,
        criminal_index: usize,
    },
}

#[derive(Default)]
pub struct TransactionExecutionOutcome {
    pub events: Vec<Event>,
}

impl TransactionExecutionOutcome {
    pub fn push_event(&mut self, event: Event) {
        self.events.push(event);
    }
}

pub type HeaderError = String;
pub type ExecuteTransactionError = ();
pub type CloseBlockError = String;

pub struct BlockOutcome {
    pub updated_validator_set: Option<CompactValidatorSet>,
    pub updated_consensus_params: Option<ConsensusParams>,
    pub events: Vec<Event>,
}

pub type ErrorCode = u32;
