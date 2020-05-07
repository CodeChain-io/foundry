// Copyright 2018, 2020 Kodebox, Inc.
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

use super::pod_account::PodAccount;
use ckey::Ed25519Public as Public;
use std::collections::BTreeMap;
use std::fmt;
use std::ops::Deref;

/// State of all accounts in the system expressed in Plain Old Data.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PodAccounts(BTreeMap<Public, PodAccount>);

impl Deref for PodAccounts {
    type Target = BTreeMap<Public, PodAccount>;

    fn deref(&self) -> &<Self as Deref>::Target {
        &self.0
    }
}

impl From<cjson::scheme::Accounts> for PodAccounts {
    fn from(s: cjson::scheme::Accounts) -> PodAccounts {
        let accounts = s
            .into_iter()
            .filter(|(_, acc)| !acc.is_empty())
            .map(|(addr, acc)| (addr.into_pubkey(), acc.into()))
            .collect();
        PodAccounts(accounts)
    }
}

impl fmt::Display for PodAccounts {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (add, acc) in &self.0 {
            writeln!(f, "{:?} => {}", add, acc)?;
        }
        Ok(())
    }
}
