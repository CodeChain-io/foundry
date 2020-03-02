// Copyright 2020 Kodebox, Inc.
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

mod log;
mod manager;
pub mod types;

pub use manager::Manager;
use types::Sequence;

/// For PoC, there is no port allocation so all channel functions will use this
const DEFAULT_PORT: &str = "DEFAULT_PORT";

pub fn channel_path(port: &str, channel: &str) -> String {
    format!("ports/{}/channels/{}", port, channel)
}

pub fn channel_capability_path(port: &str, channel: &str) -> String {
    format!("{}/key", channel_path(port, channel))
}

pub fn next_sequence_send_path(port: &str, channel: &str) -> String {
    format!("{}/nextSequenceSend", channel_path(port, channel))
}

pub fn next_sequence_recv_path(port: &str, channel: &str) -> String {
    format!("{}/nextSequenceRecv", channel_path(port, channel))
}

pub fn packet_commitment_path(port: &str, channel: &str, sequence: &Sequence) -> String {
    format!("{}/packets/{}", channel_path(port, channel), sequence.raw)
}

pub fn packet_acknowledgement_path(port: &str, channel: &str, sequence: &Sequence) -> String {
    format!("{}/acknowledgements/{}", channel_path(port, channel), sequence.raw)
}
