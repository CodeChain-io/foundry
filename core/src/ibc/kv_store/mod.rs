// Copyright 2018-2019 Kodebox, Inc.
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

use merkle_trie::proof::{CryptoProof, CryptoProofUnit};
use primitives::{Bytes, H256};

pub type Path<'a> = &'a str;

// An abstraction of state db that will be provided as a environment for the ICS handler.
pub trait KVStore {
    fn get(&self, path: Path) -> Option<Bytes>;
    fn contains_key(&self, path: Path) -> bool;
    fn insert(&mut self, path: Path, value: &[u8]) -> Option<Bytes>;
    fn remove(&mut self, path: Path) -> Option<Bytes>;
    fn root(&self) -> H256;
    fn make_proof(&self, path: Path) -> (CryptoProofUnit, CryptoProof);
}
