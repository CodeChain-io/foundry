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

use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
// Error type which should be exposed to other modules
pub enum Error {
    InsufficentBalance {
        balance: u64,
        withdrawal: u64,
    },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg: String = match *self {
            Error::InsufficentBalance {
                balance,
                withdrawal,
            } => format!("Invalid Value. The balance {} is smaller than this value {}", balance, withdrawal),
        };
        msg.fmt(f)
    }
}

// Error type which should be returned by check_transaction
pub enum CheckError {
    InsufficentBalance = 1,
    InvalidSeq = 2,
    InvalidNetworkId = 3,
    InvalidSignature = 4,
}
