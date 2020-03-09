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

use crate::CacheableItem;
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream, NULL_RLP};
use std::ops::Deref;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct IBCData(Bytes);

impl Default for IBCData {
    fn default() -> Self {
        IBCData(NULL_RLP.to_vec())
    }
}

impl Deref for IBCData {
    type Target = Bytes;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl CacheableItem for IBCData {
    type Address = H256;
    fn is_null(&self) -> bool {
        self.is_empty()
    }
}

impl Encodable for IBCData {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_raw(&self.0, 1);
    }
}

impl Decodable for IBCData {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        Ok(IBCData(rlp.as_raw().to_vec()))
    }
}

impl From<Bytes> for IBCData {
    fn from(f: Vec<u8>) -> Self {
        IBCData(f)
    }
}

impl From<IBCData> for Bytes {
    fn from(f: IBCData) -> Self {
        f.0
    }
}
