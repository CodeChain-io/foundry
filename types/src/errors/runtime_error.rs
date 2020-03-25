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
use crate::util::unexpected::Mismatch;
use crate::StorageId;
use ckey::Address;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::fmt::{Display, Formatter, Result as FormatResult};

#[derive(Debug, PartialEq, Clone, Eq, Serialize)]
#[serde(tag = "type", content = "content")]
pub enum Error {
    /// Sender doesn't have enough funds to pay for this Transaction
    InsufficientBalance {
        address: Address,
        /// Senders balance
        balance: u64,
        /// Transaction cost
        cost: u64,
    },
    InsufficientPermission,
    /// Returned when transaction seq does not match state seq
    InvalidStorageId(StorageId),
    InvalidTransferDestination,
    NewOwnersMustContainSender,
    InvalidSeq(Mismatch<u64>),
    NonActiveAccount {
        address: Address,
        name: String,
    },
    FailedToHandleCustomAction(String),
    SignatureOfInvalidAccount(Address),
    InsufficientStakes(Mismatch<u64>),
    InvalidValidatorIndex {
        idx: usize,
        parent_height: u64,
    },
}

#[derive(Clone, Copy)]
#[repr(u8)]
enum ErrorID {
    InsufficientBalance = 1,
    InsufficientPermission = 2,
    InvalidStorageID = 3,
    InvalidTransferDestination = 4,
    NewOwnersMustContainSender = 5,
    InvalidSeq = 6,
    NonActiveAccount = 7,
    FailedToHandleCustomAction = 8,
    SignatureOfInvalid = 9,
    InsufficientStakes = 10,
    InvalidValidatorIndex = 11,
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
            1u8 => Ok(ErrorID::InsufficientBalance),
            2 => Ok(ErrorID::InsufficientPermission),
            3 => Ok(ErrorID::InvalidTransferDestination),
            4 => Ok(ErrorID::NewOwnersMustContainSender),
            5 => Ok(ErrorID::InvalidSeq),
            6 => Ok(ErrorID::NonActiveAccount),
            7 => Ok(ErrorID::FailedToHandleCustomAction),
            8 => Ok(ErrorID::SignatureOfInvalid),
            9 => Ok(ErrorID::InsufficientStakes),
            10 => Ok(ErrorID::InvalidValidatorIndex),
            _ => Err(DecoderError::Custom("Unexpected ActionTag Value")),
        }
    }
}

struct RlpHelper;
impl TaggedRlp for RlpHelper {
    type Tag = ErrorID;

    fn length_of(tag: ErrorID) -> Result<usize, DecoderError> {
        Ok(match tag {
            ErrorID::FailedToHandleCustomAction => 2,
            ErrorID::InsufficientBalance => 4,
            ErrorID::InsufficientPermission => 1,
            ErrorID::InvalidSeq => 2,
            ErrorID::InvalidStorageID => 2,
            ErrorID::InvalidTransferDestination => 1,
            ErrorID::NewOwnersMustContainSender => 1,
            ErrorID::NonActiveAccount => 3,
            ErrorID::SignatureOfInvalid => 2,
            ErrorID::InsufficientStakes => 3,
            ErrorID::InvalidValidatorIndex => 3,
        })
    }
}

impl Encodable for Error {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            Error::FailedToHandleCustomAction(detail) => {
                RlpHelper::new_tagged_list(s, ErrorID::FailedToHandleCustomAction).append(detail)
            }
            Error::InsufficientBalance {
                address,
                balance,
                cost,
            } => {
                RlpHelper::new_tagged_list(s, ErrorID::InsufficientBalance).append(address).append(balance).append(cost)
            }
            Error::InsufficientPermission => RlpHelper::new_tagged_list(s, ErrorID::InsufficientPermission),
            Error::InvalidSeq(mismatch) => RlpHelper::new_tagged_list(s, ErrorID::InvalidSeq).append(mismatch),
            Error::InvalidStorageId(storage_id) => {
                RlpHelper::new_tagged_list(s, ErrorID::InvalidStorageID).append(storage_id)
            }
            Error::InvalidTransferDestination => RlpHelper::new_tagged_list(s, ErrorID::InvalidTransferDestination),
            Error::NewOwnersMustContainSender => RlpHelper::new_tagged_list(s, ErrorID::NewOwnersMustContainSender),
            Error::NonActiveAccount {
                address,
                name,
            } => RlpHelper::new_tagged_list(s, ErrorID::NonActiveAccount).append(address).append(name),
            Error::SignatureOfInvalidAccount(address) => {
                RlpHelper::new_tagged_list(s, ErrorID::SignatureOfInvalid).append(address)
            }
            Error::InsufficientStakes(Mismatch {
                expected,
                found,
            }) => RlpHelper::new_tagged_list(s, ErrorID::InsufficientStakes).append(expected).append(found),
            Error::InvalidValidatorIndex {
                idx,
                parent_height,
            } => RlpHelper::new_tagged_list(s, ErrorID::InvalidValidatorIndex).append(idx).append(parent_height),
        };
    }
}

impl Decodable for Error {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let tag = rlp.val_at(0)?;
        let error = match tag {
            ErrorID::FailedToHandleCustomAction => Error::FailedToHandleCustomAction(rlp.val_at(1)?),
            ErrorID::InsufficientBalance => Error::InsufficientBalance {
                address: rlp.val_at(1)?,
                balance: rlp.val_at(2)?,
                cost: rlp.val_at(3)?,
            },
            ErrorID::InsufficientPermission => Error::InsufficientPermission,
            ErrorID::InvalidSeq => Error::InvalidSeq(rlp.val_at(1)?),
            ErrorID::InvalidStorageID => Error::InvalidStorageId(rlp.val_at(1)?),
            ErrorID::InvalidTransferDestination => Error::InvalidTransferDestination,
            ErrorID::NewOwnersMustContainSender => Error::NewOwnersMustContainSender,
            ErrorID::NonActiveAccount => Error::NonActiveAccount {
                address: rlp.val_at(1)?,
                name: rlp.val_at(2)?,
            },
            ErrorID::SignatureOfInvalid => Error::SignatureOfInvalidAccount(rlp.val_at(1)?),
            ErrorID::InsufficientStakes => Error::InsufficientStakes(Mismatch {
                expected: rlp.val_at(1)?,
                found: rlp.val_at(2)?,
            }),
            ErrorID::InvalidValidatorIndex => Error::InvalidValidatorIndex {
                idx: rlp.val_at(1)?,
                parent_height: rlp.val_at(2)?,
            },
        };
        RlpHelper::check_size(rlp, tag)?;
        Ok(error)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> FormatResult {
        match self {
            Error::FailedToHandleCustomAction(detail) => write!(f, "Cannot handle custom action: {}", detail),
            Error::InsufficientBalance {
                address,
                balance,
                cost,
            } => write!(f, "{} has only {:?} but it must be larger than {:?}", address, balance, cost),
            Error::InsufficientPermission => write!(f, "Sender doesn't have a permission"),
            Error::InvalidSeq(mismatch) => write!(f, "Invalid transaction seq {}", mismatch),
            Error::InvalidStorageId(storage_id) => write!(f, "{} is an invalid storage id", storage_id),
            Error::InvalidTransferDestination => write!(f, "Transfer receiver is not valid account"),
            Error::NewOwnersMustContainSender => write!(f, "New owners must contain the sender"),
            Error::NonActiveAccount {
                name,
                address,
            } => write!(f, "Non active account({}) cannot be {}", address, name),
            Error::SignatureOfInvalidAccount(address) => {
                write!(f, "Signature of invalid account({}) received", address)
            }
            Error::InsufficientStakes(mismatch) => write!(f, "Insufficient stakes: {}", mismatch),
            Error::InvalidValidatorIndex {
                idx,
                parent_height,
            } => write!(f, "The validator index {} is invalid at the parent hash {}", idx, parent_height),
        }
    }
}
