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

use super::{CompactValidatorSet, Header};
use std::ops::Deref;

#[derive(Clone, Debug, RlpEncodable, RlpDecodable)]
pub struct SyncHeader {
    block_header: Header,
    prev_validator_set: Option<CompactValidatorSet>,
}

impl SyncHeader {
    pub fn new(block_header: Header, validator_set: Option<CompactValidatorSet>) -> Self {
        Self {
            block_header,
            prev_validator_set: validator_set,
        }
    }
}

impl Deref for SyncHeader {
    type Target = Header;
    fn deref(&self) -> &Self::Target {
        &self.block_header
    }
}

impl From<SyncHeader> for Header {
    fn from(sync_header: SyncHeader) -> Self {
        sync_header.block_header
    }
}
