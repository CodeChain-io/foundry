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

use super::ConsensusEngine;
use crate::codechain_machine::CodeChainMachine;
use crate::consensus::{EngineError, EngineType};
use ckey::Address;

/// An engine which does not provide any consensus mechanism and does not seal blocks.
pub struct NullEngine {
    machine: CodeChainMachine,
}

impl NullEngine {
    /// Returns new instance of NullEngine with default VM Factory
    pub fn new(machine: CodeChainMachine) -> Self {
        NullEngine {
            machine,
        }
    }
}

impl ConsensusEngine for NullEngine {
    fn name(&self) -> &str {
        "NullEngine"
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

    fn possible_authors(&self, _block_number: Option<u64>) -> Result<Option<Vec<Address>>, EngineError> {
        Ok(None)
    }
}
