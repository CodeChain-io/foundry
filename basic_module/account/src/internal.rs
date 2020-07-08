// Copyright 2020 Kodebox, Inc.
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

use crate::error::Error;
use crate::types::Account;
use ckey::Ed25519Public as Public;
use coordinator::context::Context;

pub fn add_balance(context: &mut dyn Context, account_id: &Public, val: u64) {
    if val == 0 {
        return
    }

    let mut account: Account = get_account(context, account_id);

    account.balance += val;
    context.set(account_id.as_ref(), account.to_vec());
}

pub fn sub_balance(context: &mut dyn Context, account_id: &Public, val: u64) -> Result<(), Error> {
    let mut account: Account = get_account(context, account_id);

    if account.balance < val {
        return Err(Error::InvalidValue(account.balance, val))
    }

    account.balance -= val;
    context.set(account_id.as_ref(), account.to_vec());
    Ok(())
}

pub fn get_sequence(context: &dyn Context, account_id: &Public) -> u64 {
    get_account(context, account_id).sequence
}

pub fn get_balance(context: &dyn Context, account_id: &Public) -> u64 {
    get_account(context, account_id).balance
}

pub fn get_account(context: &dyn Context, account_id: &Public) -> Account {
    context.get(account_id.as_ref()).map(|account| account.into()).unwrap_or_default()
}
