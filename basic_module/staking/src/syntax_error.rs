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

use crate::types::{NetworkId, Signature};
use std::fmt;

#[derive(Debug)]
pub enum Error {
    InvalidSignature(Signature),
    InvalidNetworkId(NetworkId),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidNetworkId(network_id) => write!(f, "{} is an invalid network id", network_id),
            Error::InvalidSignature(sig) => write!(f, "Signature {:?} is invalid", sig),
        }
    }
}

impl Error {
    pub fn code(&self) -> i64 {
        match self {
            Error::InvalidSignature(_) => -1,
            Error::InvalidNetworkId(_) => -2,
        }
    }
}
