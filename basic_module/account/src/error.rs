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

use ckey::{NetworkId, Signature};
use std::fmt;

#[allow(dead_code)]
#[derive(Debug)]
pub enum Error {
    InvalidValue(u64, u64),
    InvalidSignature(Signature),
    InvalidNetworkId(NetworkId),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let msg: String = match *self {
            Error::InvalidValue(balance, value) => {
                format!("Invalid Value. The balance {} is smaller than this value {}", balance, value)
            }
            Error::InvalidNetworkId(network_id) => format!("{} is an invalid network id", network_id),
            Error::InvalidSignature(sig) => format!("Signature {:?} is invalid", sig),
        };

        msg.fmt(f)
    }
}
