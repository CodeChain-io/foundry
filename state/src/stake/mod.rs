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

mod actions;

pub use self::actions::{ban, init_stake, revert_delegations};
use super::TopStateView;
use crate::{StateResult, TopLevelState};
use ccrypto::blake256;
use ckey::Ed25519Public as Public;
use ctypes::errors::SyntaxError;
use primitives::H256;
use rlp::{Encodable, RlpStream};
use std::convert::From;

pub trait DoubleVoteHandler: Send + Sync {
    fn execute(&self, message1: &[u8], state: &mut TopLevelState, fee_payer: &Public) -> StateResult<()>;
    fn verify(&self, message1: &[u8], message2: &[u8]) -> Result<(), SyntaxError>;
}

pub fn query(key_fragment: &[u8], state: &TopLevelState) -> StateResult<Option<Vec<u8>>> {
    let key = StakeKeyBuilder::key_from_fragment(key_fragment);
    let some_action_data = state.action_data(&key)?.map(Vec::from);
    Ok(some_action_data)
}

pub struct StakeKeyBuilder {
    rlp: RlpStream,
}

impl StakeKeyBuilder {
    fn prepare() -> StakeKeyBuilder {
        let mut rlp = RlpStream::new_list(2);
        rlp.append(&"Stake");
        StakeKeyBuilder {
            rlp,
        }
    }

    pub fn key_from_fragment(key_fragment: &[u8]) -> H256 {
        let mut builder = Self::prepare();
        builder.rlp.append_raw(&key_fragment, 1);
        builder.into_key()
    }

    pub fn new(fragment_length: usize) -> StakeKeyBuilder {
        let mut builder = Self::prepare();
        builder.rlp.begin_list(fragment_length);
        builder
    }

    pub fn append<E>(mut self, e: &E) -> StakeKeyBuilder
    where
        E: Encodable, {
        self.rlp.append(e);
        self
    }

    pub fn into_key(self) -> H256 {
        blake256(self.rlp.as_raw())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn action_data_key_builder_raw_fragment_and_list_are_same() {
        let key1 = StakeKeyBuilder::new(3).append(&"key").append(&"fragment").append(&"has trailing list").into_key();

        let mut rlp = RlpStream::new_list(3);
        rlp.append(&"key").append(&"fragment").append(&"has trailing list");
        let key2 = StakeKeyBuilder::key_from_fragment(rlp.as_raw());
        assert_eq!(key1, key2);
    }
}
