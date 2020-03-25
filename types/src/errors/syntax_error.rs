// Copyright 2019-2020. Kodebox, Inc.
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

use super::TaggedRlp;
use ckey::NetworkId;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug, PartialEq, Clone, Eq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum Error {
    /// Transaction's fee is below currently set minimal fee requirement.
    InsufficientFee {
        /// Minimal expected fee
        minimal: u64,
        /// Transaction fee
        got: u64,
    },
    InvalidCustomAction(String),
    /// Invalid network ID given.
    InvalidNetworkId(NetworkId),
    InvalidApproval(String),
    /// Max metadata size is exceeded.
    MetadataTooBig,
    TextContentTooBig,
    TransactionIsTooBig,
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum ErrorID {
    InsufficientFee = 1,
    InvalidNetworkID = 2,
    InvalidApproval = 3,
    MetadataTooBig = 4,
    TextContentTooBig = 5,
    TxIsTooBig = 6,
    InvalidCustomAction = 7,
}

impl Encodable for ErrorID {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(*self as u8));
    }
}

impl Decodable for ErrorID {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let tag = rlp.as_val()?;
        match tag {
            1u8 => Ok(ErrorID::InsufficientFee),
            2 => Ok(ErrorID::InvalidNetworkID),
            3 => Ok(ErrorID::InvalidApproval),
            4 => Ok(ErrorID::MetadataTooBig),
            5 => Ok(ErrorID::TextContentTooBig),
            6 => Ok(ErrorID::TxIsTooBig),
            7 => Ok(ErrorID::InvalidCustomAction),
            _ => Err(DecoderError::Custom("Unexpected ErrorID Value")),
        }
    }
}

struct RlpHelper;
impl TaggedRlp for RlpHelper {
    type Tag = ErrorID;

    fn length_of(tag: ErrorID) -> Result<usize, DecoderError> {
        Ok(match tag {
            ErrorID::InsufficientFee => 3,
            ErrorID::InvalidCustomAction => 2,
            ErrorID::InvalidNetworkID => 2,
            ErrorID::InvalidApproval => 2,
            ErrorID::MetadataTooBig => 1,
            ErrorID::TextContentTooBig => 1,
            ErrorID::TxIsTooBig => 1,
        })
    }
}

impl Encodable for Error {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            Error::InsufficientFee {
                minimal,
                got,
            } => RlpHelper::new_tagged_list(s, ErrorID::InsufficientFee).append(minimal).append(got),
            Error::InvalidCustomAction(err) => RlpHelper::new_tagged_list(s, ErrorID::InvalidCustomAction).append(err),
            Error::InvalidNetworkId(network_id) => {
                RlpHelper::new_tagged_list(s, ErrorID::InvalidNetworkID).append(network_id)
            }
            Error::InvalidApproval(err) => RlpHelper::new_tagged_list(s, ErrorID::InvalidApproval).append(err),
            Error::MetadataTooBig => RlpHelper::new_tagged_list(s, ErrorID::MetadataTooBig),
            Error::TextContentTooBig => RlpHelper::new_tagged_list(s, ErrorID::TextContentTooBig),
            Error::TransactionIsTooBig => RlpHelper::new_tagged_list(s, ErrorID::TxIsTooBig),
        };
    }
}

impl Decodable for Error {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let tag = rlp.val_at(0)?;
        let error = match tag {
            ErrorID::InsufficientFee => Error::InsufficientFee {
                minimal: rlp.val_at(1)?,
                got: rlp.val_at(2)?,
            },
            ErrorID::InvalidCustomAction => Error::InvalidCustomAction(rlp.val_at(1)?),
            ErrorID::InvalidNetworkID => Error::InvalidNetworkId(rlp.val_at(1)?),
            ErrorID::InvalidApproval => Error::InvalidApproval(rlp.val_at(1)?),
            ErrorID::MetadataTooBig => Error::MetadataTooBig,
            ErrorID::TextContentTooBig => Error::TextContentTooBig,
            ErrorID::TxIsTooBig => Error::TransactionIsTooBig,
        };
        RlpHelper::check_size(rlp, tag)?;
        Ok(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            Error::InsufficientFee {
                minimal,
                got,
            } => write!(f, "Insufficient fee. Min={}, Given={}", minimal, got),
            Error::InvalidCustomAction(err) => write!(f, "Invalid custom action: {}", err),
            Error::InvalidNetworkId(network_id) => write!(f, "{} is an invalid network id", network_id),
            Error::InvalidApproval(err) => write!(f, "Transaction has an invalid approval :{}", err),
            Error::MetadataTooBig => write!(f, "Metadata size is too big."),
            Error::TextContentTooBig => write!(f, "The content of the text is too big"),
            Error::TransactionIsTooBig => write!(f, "Transaction size exceeded the body size limit"),
        }
    }
}
