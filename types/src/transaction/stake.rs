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

use crate::errors::SyntaxError;
use crate::CommonParams;
use ccrypto::Blake;
use ckey::{verify, Address, Ed25519Public as Public, Signature};
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Copy)]
#[repr(u8)]
enum StakeActionTag {
    TransferCCS = 1,
    DelegateCCS = 2,
    Revoke = 3,
    SelfNominate = 4,
    ReportDoubleVote = 5,
    Redelegate = 6,
    ChangeParams = 0xFF,
}

impl Encodable for StakeActionTag {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(*self as u8));
    }
}

impl Decodable for StakeActionTag {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let tag = rlp.as_val()?;
        match tag {
            1u8 => Ok(StakeActionTag::TransferCCS),
            2 => Ok(StakeActionTag::DelegateCCS),
            3 => Ok(StakeActionTag::Revoke),
            4 => Ok(StakeActionTag::SelfNominate),
            5 => Ok(StakeActionTag::ReportDoubleVote),
            6 => Ok(StakeActionTag::Redelegate),
            0xFF => Ok(StakeActionTag::ChangeParams),
            _ => Err(DecoderError::Custom("Unexpected ActionTag Value")),
        }
    }
}

#[derive(Debug, PartialEq, RlpEncodable, RlpDecodable)]
pub struct Approval {
    signature: Signature,
    signer_public: Public,
}

impl Approval {
    pub fn signature(&self) -> &Signature {
        &self.signature
    }

    pub fn signer_public(&self) -> &Public {
        &self.signer_public
    }
}

#[derive(Debug, PartialEq)]
pub enum StakeAction {
    TransferCCS {
        address: Address,
        quantity: u64,
    },
    DelegateCCS {
        address: Address,
        quantity: u64,
    },
    Revoke {
        address: Address,
        quantity: u64,
    },
    Redelegate {
        prev_delegatee: Address,
        next_delegatee: Address,
        quantity: u64,
    },
    SelfNominate {
        deposit: u64,
        metadata: Bytes,
    },
    ChangeParams {
        metadata_seq: u64,
        params: Box<CommonParams>,
        approvals: Vec<Approval>,
    },
    ReportDoubleVote {
        message1: Bytes,
        message2: Bytes,
    },
}

impl StakeAction {
    pub fn verify(&self, current_params: &CommonParams) -> Result<(), SyntaxError> {
        match self {
            StakeAction::TransferCCS {
                ..
            } => {}
            StakeAction::DelegateCCS {
                ..
            } => {}
            StakeAction::Revoke {
                ..
            } => {}
            StakeAction::Redelegate {
                ..
            } => {}
            StakeAction::SelfNominate {
                metadata,
                ..
            } => {
                if metadata.len() > current_params.max_candidate_metadata_size() {
                    return Err(SyntaxError::InvalidCustomAction(format!(
                        "Too long candidate metadata: the size limit is {}",
                        current_params.max_candidate_metadata_size()
                    )))
                }
            }
            StakeAction::ChangeParams {
                metadata_seq,
                params,
                approvals,
            } => {
                params.verify_change(current_params).map_err(SyntaxError::InvalidCustomAction)?;
                let action = StakeAction::ChangeParams {
                    metadata_seq: *metadata_seq,
                    params: params.clone(),
                    approvals: vec![],
                };
                let encoded_action = H256::blake(rlp::encode(&action));
                for approval in approvals {
                    if !verify(approval.signature(), &encoded_action, approval.signer_public()) {
                        return Err(SyntaxError::InvalidCustomAction(format!(
                            "Cannot decode the signature {:?} with public {:?} and the message {:?}",
                            approval.signature(),
                            approval.signer_public(),
                            &encoded_action,
                        )))
                    }
                }
            }
            StakeAction::ReportDoubleVote {
                message1,
                message2,
            } => {
                if message1 == message2 {
                    return Err(SyntaxError::InvalidCustomAction(String::from("Messages are duplicated")))
                }
            }
        }
        Ok(())
    }
}

impl Encodable for StakeAction {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            StakeAction::TransferCCS {
                address,
                quantity,
            } => {
                s.begin_list(3).append(&StakeActionTag::TransferCCS).append(address).append(quantity);
            }
            StakeAction::DelegateCCS {
                address,
                quantity,
            } => {
                s.begin_list(3).append(&StakeActionTag::DelegateCCS).append(address).append(quantity);
            }
            StakeAction::Revoke {
                address,
                quantity,
            } => {
                s.begin_list(3).append(&StakeActionTag::Revoke).append(address).append(quantity);
            }
            StakeAction::Redelegate {
                prev_delegatee,
                next_delegatee,
                quantity,
            } => {
                s.begin_list(4)
                    .append(&StakeActionTag::Redelegate)
                    .append(prev_delegatee)
                    .append(next_delegatee)
                    .append(quantity);
            }
            StakeAction::SelfNominate {
                deposit,
                metadata,
            } => {
                s.begin_list(3).append(&StakeActionTag::SelfNominate).append(deposit).append(metadata);
            }
            StakeAction::ChangeParams {
                metadata_seq,
                params,
                approvals,
            } => {
                s.begin_list(3 + approvals.len())
                    .append(&StakeActionTag::ChangeParams)
                    .append(metadata_seq)
                    .append(&**params);
                for approval in approvals {
                    s.append(approval);
                }
            }
            StakeAction::ReportDoubleVote {
                message1,
                message2,
            } => {
                s.begin_list(3).append(&StakeActionTag::ReportDoubleVote).append(message1).append(message2);
            }
        };
    }
}

impl Decodable for StakeAction {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let tag = rlp.val_at(0)?;
        match tag {
            StakeActionTag::TransferCCS => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 3,
                        got: item_count,
                    })
                }
                Ok(StakeAction::TransferCCS {
                    address: rlp.val_at(1)?,
                    quantity: rlp.val_at(2)?,
                })
            }
            StakeActionTag::DelegateCCS => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 3,
                        got: item_count,
                    })
                }
                Ok(StakeAction::DelegateCCS {
                    address: rlp.val_at(1)?,
                    quantity: rlp.val_at(2)?,
                })
            }
            StakeActionTag::Revoke => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 3,
                        got: item_count,
                    })
                }
                Ok(StakeAction::Revoke {
                    address: rlp.val_at(1)?,
                    quantity: rlp.val_at(2)?,
                })
            }
            StakeActionTag::Redelegate => {
                let item_count = rlp.item_count()?;
                if item_count != 4 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 4,
                        got: item_count,
                    })
                }
                Ok(StakeAction::Redelegate {
                    prev_delegatee: rlp.val_at(1)?,
                    next_delegatee: rlp.val_at(2)?,
                    quantity: rlp.val_at(3)?,
                })
            }
            StakeActionTag::SelfNominate => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 3,
                        got: item_count,
                    })
                }
                Ok(StakeAction::SelfNominate {
                    deposit: rlp.val_at(1)?,
                    metadata: rlp.val_at(2)?,
                })
            }
            StakeActionTag::ChangeParams => {
                let item_count = rlp.item_count()?;
                if item_count < 4 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        expected: 4,
                        got: item_count,
                    })
                }
                let metadata_seq = rlp.val_at(1)?;
                let params = Box::new(rlp.val_at(2)?);
                let approvals = (3..item_count).map(|i| rlp.val_at(i)).collect::<Result<_, _>>()?;
                Ok(StakeAction::ChangeParams {
                    metadata_seq,
                    params,
                    approvals,
                })
            }
            StakeActionTag::ReportDoubleVote => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        expected: 3,
                        got: item_count,
                    })
                }
                let message1 = rlp.val_at(1)?;
                let message2 = rlp.val_at(2)?;
                Ok(StakeAction::ReportDoubleVote {
                    message1,
                    message2,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn rlp_of_change_params() {
        rlp_encode_and_decode_test!(StakeAction::ChangeParams {
            metadata_seq: 3,
            params: CommonParams::default_for_test().into(),
            approvals: vec![
                Approval {
                    signature: Signature::random(),
                    signer_public: Public::random(),
                },
                Approval {
                    signature: Signature::random(),
                    signer_public: Public::random(),
                }
            ],
        });
    }

    #[test]
    fn decode_fail_if_change_params_have_no_signatures() {
        let action = StakeAction::ChangeParams {
            metadata_seq: 3,
            params: CommonParams::default_for_test().into(),
            approvals: vec![],
        };
        assert_eq!(
            Err(DecoderError::RlpIncorrectListLen {
                expected: 4,
                got: 3,
            }),
            Rlp::new(&rlp::encode(&action)).as_val::<StakeAction>()
        );
    }
}
