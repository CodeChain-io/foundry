// Copyright 2015-2017 Parity Technologies (UK) Ltd.
// This file is part of Parity.

// Parity is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Parity is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Parity.  If not, see <http://www.gnu.org/licenses/>.

use crate::accounts_dir::{DiskKeyFileManager, KeyDirectory, KeyFileManager};
use crate::Error;
use ckey::Ed25519Public as Public;
use std::collections::HashSet;
use std::ffi::OsStr;
use std::fs;
use std::path::Path;

/// Import an account from a file.
pub fn import_account(path: &Path, dst: &dyn KeyDirectory) -> Result<Public, Error> {
    let key_manager = DiskKeyFileManager;
    let existing_accounts = dst.load()?.into_iter().map(|a| a.pubkey).collect::<HashSet<_>>();
    let filename = path.file_name().and_then(OsStr::to_str).map(ToOwned::to_owned);
    let account = fs::File::open(&path).map_err(Into::into).and_then(|file| key_manager.read(filename, file))?;

    let pubkey = account.pubkey;
    if !existing_accounts.contains(&pubkey) {
        dst.insert(account)?;
    }
    Ok(pubkey)
}

/// Import all accounts from one directory to the other.
pub fn import_accounts(src: &dyn KeyDirectory, dst: &dyn KeyDirectory) -> Result<Vec<Public>, Error> {
    let accounts = src.load()?;
    let existing_accounts = dst.load()?.into_iter().map(|a| a.pubkey).collect::<HashSet<_>>();

    accounts
        .into_iter()
        .filter(|a| !existing_accounts.contains(&a.pubkey))
        .map(|a| {
            let address = a.pubkey;
            dst.insert(a)?;
            Ok(address)
        })
        .collect()
}
