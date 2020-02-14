// Copyright 2018 Kodebox, Inc.
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

use cjson;
use cstate::Account;
use rlp::{Encodable, RlpStream};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
/// An account, expressed as Plain-Old-Data (hence the name).
/// Does not have a DB overlay cache, code hash or anything like that.
pub struct PodAccount {
    /// The balance of the account.
    pub balance: u64,
    /// The seq of the account.
    pub seq: u64,
}

impl<'a> From<&'a PodAccount> for Account {
    fn from(pod: &'a PodAccount) -> Self {
        Account::new(pod.balance, pod.seq)
    }
}

impl Encodable for PodAccount {
    fn rlp_append(&self, stream: &mut RlpStream) {
        let account: Account = self.into();
        account.rlp_append(stream);
    }
}

impl From<cjson::scheme::Account> for PodAccount {
    fn from(a: cjson::scheme::Account) -> Self {
        PodAccount {
            balance: a.balance.map_or(0, Into::into),
            seq: a.seq.map_or(0, Into::into),
        }
    }
}

impl fmt::Display for PodAccount {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "(bal={}; seq={})", self.balance, self.seq,)
    }
}
