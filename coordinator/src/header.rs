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

use ckey::Ed25519Public as Public;
use primitives::Bytes;

#[derive(Debug, Clone, PartialEq)]
pub struct Header {
    /// Block timestamp.
    timestamp: u64,
    /// Block number.
    number: u64,
    /// Block author.
    author: Public,
    /// Block extra data.
    extra_data: Bytes,
}

impl Header {
    #[allow(dead_code)]
    pub fn new(timestamp: u64, number: u64, author: Public, extra_data: Bytes) -> Self {
        Self {
            timestamp,
            number,
            author,
            extra_data,
        }
    }
}
