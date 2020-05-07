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

use crate::get_context;
use crate::types::Account;
use ckey::Ed25519Public as Public;

pub fn get_sequence(account_id: &Public) -> u64 {
    get_account(account_id).sequence
}

pub fn get_account(account_id: &Public) -> Account {
    get_context().get(account_id).map(|account| account.into()).unwrap_or_default()
}
