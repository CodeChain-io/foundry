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

use super::Action;
use crate::TxHash;
use ccrypto::blake256;
use ckey::NetworkId;
use rlp::RlpStream;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Network Id
    pub network_id: NetworkId,

    pub action: Action,
}

impl Transaction {
    /// Append object with a without signature into RLP stream
    pub fn rlp_append_unsigned(&self, s: &mut RlpStream) {
        s.begin_list(2);
        s.append(&self.network_id);
        s.append(&self.action);
    }

    /// The message hash of the transaction.
    pub fn hash(&self) -> TxHash {
        let mut stream = RlpStream::new();
        self.rlp_append_unsigned(&mut stream);
        blake256(stream.as_raw()).into()
    }
}
