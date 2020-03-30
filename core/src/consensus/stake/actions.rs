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

use crate::client::ConsensusClient;
use crate::consensus::{ConsensusMessage, ValidatorSet};
use ccrypto::Blake;
use ckey::{verify, Address, Ed25519Public as Public, Signature};
use ctypes::errors::SyntaxError;
use ctypes::CommonParams;
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use std::sync::Arc;

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
    pub fn verify(
        &self,
        current_params: &CommonParams,
        client: Option<Arc<dyn ConsensusClient>>,
        validators: Option<Arc<dyn ValidatorSet>>,
    ) -> Result<(), SyntaxError> {
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
                let message1: ConsensusMessage =
                    rlp::decode(&message1).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;
                let message2: ConsensusMessage =
                    rlp::decode(&message2).map_err(|err| SyntaxError::InvalidCustomAction(err.to_string()))?;
                if message1.round() != message2.round() {
                    return Err(SyntaxError::InvalidCustomAction(String::from(
                        "The messages are from two different voting rounds",
                    )))
                }

                let signer_idx1 = message1.signer_index();
                let signer_idx2 = message2.signer_index();

                if signer_idx1 != signer_idx2 {
                    return Err(SyntaxError::InvalidCustomAction(format!(
                        "Two messages have different signer indexes: {}, {}",
                        signer_idx1, signer_idx2
                    )))
                }

                assert_eq!(
                    message1.height(),
                    message2.height(),
                    "Heights of both messages must be same because message1.round() == message2.round()"
                );
                let signed_block_height = message1.height();
                let (client, validators) = (
                    client.expect("Client should be initialized"),
                    validators.expect("ValidatorSet should be initialized"),
                );
                if signed_block_height == 0 {
                    return Err(SyntaxError::InvalidCustomAction(String::from(
                        "Double vote on the genesis block does not make sense",
                    )))
                }
                let parent_hash = client
                    .block_header(&(signed_block_height - 1).into())
                    .ok_or_else(|| {
                        SyntaxError::InvalidCustomAction(format!(
                            "Cannot get header from the height {}",
                            signed_block_height
                        ))
                    })?
                    .hash();
                let signer = validators.get(&parent_hash, signer_idx1);
                if !message1.verify(&signer) || !message2.verify(&signer) {
                    return Err(SyntaxError::InvalidCustomAction(String::from("Ed25519 signature verification fails")))
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
    use crate::client::TestBlockChainClient;
    use crate::consensus::{ConsensusMessage, DynamicValidator, Step, VoteOn, VoteStep};
    use ckey::sign;
    use ctypes::BlockHash;
    use rlp::rlp_encode_and_decode_test;

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

    struct ConsensusMessageInfo {
        pub height: u64,
        pub view: u64,
        pub step: Step,
        pub block_hash: Option<BlockHash>,
        pub signer_index: usize,
    }

    fn create_consensus_message<F, G>(
        info: ConsensusMessageInfo,
        client: &TestBlockChainClient,
        vote_step_twister: &F,
        block_hash_twister: &G,
    ) -> ConsensusMessage
    where
        F: Fn(VoteStep) -> VoteStep,
        G: Fn(Option<BlockHash>) -> Option<BlockHash>, {
        let ConsensusMessageInfo {
            height,
            view,
            step,
            block_hash,
            signer_index,
        } = info;
        let vote_step = VoteStep::new(height, view, step);
        let on = VoteOn {
            step: vote_step,
            block_hash,
        };
        let twisted = VoteOn {
            step: vote_step_twister(vote_step),
            block_hash: block_hash_twister(block_hash),
        };
        let reversed_idx = client.get_validators().len() - 1 - signer_index;
        let pubkey = *client.get_validators().get(reversed_idx).unwrap().pubkey();
        let validator_keys = client.validator_keys.read();
        let privkey = validator_keys.get(&pubkey).unwrap();
        let signature = sign(&twisted.hash(), privkey);

        ConsensusMessage {
            signature,
            signer_index,
            on,
        }
    }

    fn double_vote_verification_result<F, G>(
        message_info1: ConsensusMessageInfo,
        message_info2: ConsensusMessageInfo,
        vote_step_twister: &F,
        block_hash_twister: &G,
    ) -> Result<(), SyntaxError>
    where
        F: Fn(VoteStep) -> VoteStep,
        G: Fn(Option<BlockHash>) -> Option<BlockHash>, {
        let mut test_client = TestBlockChainClient::default();
        test_client.add_blocks(10, 1);
        test_client.set_random_validators(10);
        let validator_set = DynamicValidator::default();

        let consensus_message1 =
            create_consensus_message(message_info1, &test_client, vote_step_twister, block_hash_twister);
        let consensus_message2 =
            create_consensus_message(message_info2, &test_client, vote_step_twister, block_hash_twister);
        let action = StakeAction::ReportDoubleVote {
            message1: consensus_message1.rlp_bytes(),
            message2: consensus_message2.rlp_bytes(),
        };
        let arced_client: Arc<dyn ConsensusClient> = Arc::new(test_client);
        validator_set.register_client(Arc::downgrade(&arced_client));
        action.verify(&CommonParams::default_for_test(), Some(Arc::clone(&arced_client)), Some(Arc::new(validator_set)))
    }

    #[test]
    fn double_vote_verify_desirable_report() {
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: None,
                signer_index: 0,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: Some(H256::random().into()),
                signer_index: 0,
            },
            &|v| v,
            &|v| v,
        );
        assert!(result.is_ok());
    }

    #[test]
    fn double_vote_verify_same_message() {
        let block_hash = Some(H256::random().into());
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 3,
                view: 1,
                step: Step::Precommit,
                block_hash,
                signer_index: 2,
            },
            ConsensusMessageInfo {
                height: 3,
                view: 1,
                step: Step::Precommit,
                block_hash,
                signer_index: 2,
            },
            &|v| v,
            &|v| v,
        );
        let expected_err = Err(SyntaxError::InvalidCustomAction(String::from("Messages are duplicated")));
        assert_eq!(result, expected_err);
    }

    #[test]
    fn double_vote_verify_different_height() {
        let block_hash = Some(H256::random().into());
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 3,
                view: 1,
                step: Step::Precommit,
                block_hash,
                signer_index: 2,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 1,
                step: Step::Precommit,
                block_hash,
                signer_index: 2,
            },
            &|v| v,
            &|v| v,
        );
        let expected_err =
            Err(SyntaxError::InvalidCustomAction(String::from("The messages are from two different voting rounds")));
        assert_eq!(result, expected_err);
    }

    #[test]
    fn double_vote_verify_different_signer() {
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: None,
                signer_index: 1,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: Some(H256::random().into()),
                signer_index: 0,
            },
            &|v| v,
            &|v| v,
        );
        match result {
            Err(SyntaxError::InvalidCustomAction(ref s))
                if s.contains("Two messages have different signer indexes") => {}
            _ => panic!(),
        }
    }

    #[test]
    fn double_vote_verify_different_message_and_signer() {
        let hash1 = Some(H256::random().into());
        let mut hash2 = Some(H256::random().into());
        while hash1 == hash2 {
            hash2 = Some(H256::random().into());
        }
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: hash1,
                signer_index: 1,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: hash2,
                signer_index: 0,
            },
            &|v| v,
            &|v| v,
        );
        match result {
            Err(SyntaxError::InvalidCustomAction(ref s))
                if s.contains("Two messages have different signer indexes") => {}
            _ => panic!(),
        }
    }

    #[test]
    fn double_vote_verify_strange_sig1() {
        let vote_step_twister = |original: VoteStep| VoteStep {
            height: original.height + 1,
            view: original.height + 1,
            step: original.step,
        };
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: None,
                signer_index: 0,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: Some(H256::random().into()),
                signer_index: 0,
            },
            &vote_step_twister,
            &|v| v,
        );
        let expected_err = Err(SyntaxError::InvalidCustomAction(String::from("Ed25519 signature verification fails")));
        assert_eq!(result, expected_err);
    }

    #[test]
    fn double_vote_verify_strange_sig2() {
        let block_hash_twister = |original: Option<BlockHash>| {
            original.map(|hash| {
                let mut twisted = H256::random();
                while twisted == *hash {
                    twisted = H256::random();
                }
                BlockHash::from(twisted)
            })
        };
        let result = double_vote_verification_result(
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: None,
                signer_index: 0,
            },
            ConsensusMessageInfo {
                height: 2,
                view: 0,
                step: Step::Precommit,
                block_hash: Some(H256::random().into()),
                signer_index: 0,
            },
            &|v| v,
            &block_hash_twister,
        );
        let expected_err = Err(SyntaxError::InvalidCustomAction(String::from("Ed25519 signature verification fails")));
        assert_eq!(result, expected_err);
    }
}
