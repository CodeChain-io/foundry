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

pub const PREFIX: &str = "VoteBox";

pub mod common;
pub mod general_meeting;
pub mod voting;

pub fn generate_voting_box_key(meeting_id: &general_meeting::GeneralMeetingId) -> Vec<u8> {
    let byte_prefix = PREFIX.as_bytes();
    let mut vec: Vec<u8> = Vec::new();
    vec.extend_from_slice(byte_prefix);
    vec.extend_from_slice(meeting_id.id.0.as_ref());
    vec
}
