// Copyright 2019-2020 Kodebox, Inc.
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

pub mod client;
mod manager;
pub mod types;
pub use self::manager::Manager;
pub use self::types::{ConsensusState, Header, Kind, KIND_FOUNDRY};

pub fn path_client_state(id: &str) -> String {
    format!("clients/{}", id)
}

pub fn path_consensus_state(id: &str, block_number: ctypes::BlockNumber) -> String {
    format!("{}/consensusState/{}", path_client_state(id), block_number)
}
