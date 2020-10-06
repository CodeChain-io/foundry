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

use crate::{StateResult, TopLevelState};
use ccrypto::blake256;
use ckey::Ed25519Public as Public;
use ctypes::errors::SyntaxError;
use primitives::H256;
use rlp::{Encodable, RlpStream};

pub trait DoubleVoteHandler: Send + Sync {
    fn execute(&self, message1: &[u8], state: &mut TopLevelState, fee_payer: &Public) -> StateResult<()>;
    fn verify(&self, message1: &[u8], message2: &[u8]) -> Result<(), SyntaxError>;
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
