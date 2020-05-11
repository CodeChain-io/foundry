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

use crate::error::Error;
use crate::get_context;
use crate::types::Account;
use ckey::Ed25519Public as Public;

#[allow(dead_code)]
pub fn add_balance(address: &Public, val: u64) {
    if val == 0 {
        return
    }

    let context = get_context();
    let mut account: Account = get_account(address);

    account.balance += val;
    context.set(address, account.to_vec());
}

#[allow(dead_code)]
pub fn sub_balance(address: &Public, val: u64) -> Result<(), Error> {
    let context = get_context();
    let mut account: Account = get_account(address);

    if account.balance < val {
        return Err(Error::InvalidValue(account.balance, val))
    }

    account.balance -= val;
    context.set(address, account.to_vec());
    Ok(())
}

#[allow(dead_code)]
pub fn get_sequence(address: &Public) -> u64 {
    get_account(address).sequence
}

#[allow(dead_code)]
pub fn get_balance(address: &Public) -> u64 {
    get_account(address).balance
}

pub fn get_account(address: &Public) -> Account {
    get_context().get(address).map(|account| account.into()).unwrap_or_default()
}
