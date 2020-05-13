// Copyright 2018-2020 Kodebox, Inc.
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

use crate::NetworkId;
use rlp::DecoderError;
use std::fmt;

#[derive(Debug, PartialEq)]
pub enum Error {
    InvalidPublic(String),
    InvalidPrivate(String),
    InvalidSecret,
    InvalidSignature,
    InvalidNetworkId(NetworkId),
    InvalidPlatformAddressVersion(u8),
    InvalidPlatformAddressFormat(String),
    RlpDecoderError(DecoderError),
    Custom(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::InvalidPublic(key) => write!(f, "{} is an invalid public key", key),
            Error::InvalidPrivate(key) => write!(f, "{} is an invlaid private key", key),
            Error::InvalidSecret => write!(f, "Invalid Secret"),
            Error::InvalidSignature => write!(f, "Invalid Signature"),
            Error::InvalidNetworkId(network_id) => write!(f, "{} is an invalid network id", network_id),
            Error::InvalidPlatformAddressVersion(version) => {
                write!(f, "{} is an invalid platform address version", version)
            }
            Error::InvalidPlatformAddressFormat(address) => write!(f, "{} is an invalid platform string", address),
            Error::RlpDecoderError(err) => write!(f, "{}", err),

            Error::Custom(ref s) => write!(f, "{}", s),
        }
    }
}

impl From<DecoderError> for Error {
    fn from(err: DecoderError) -> Self {
        Error::RlpDecoderError(err)
    }
}
