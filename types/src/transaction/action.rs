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
use crate::{CommonParams, Tracker};
use ccrypto::Blake;
use ckey::{Address, NetworkId};
use primitives::{Bytes, H256};
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[derive(Clone, Copy)]
#[repr(u8)]
enum ActionTag {
    Pay = 0x02,
    Custom = 0xFF,
}

impl Encodable for ActionTag {
    fn rlp_append(&self, s: &mut RlpStream) {
        (*self as u8).rlp_append(s)
    }
}

impl Decodable for ActionTag {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let tag = rlp.as_val()?;
        match tag {
            0x02u8 => Ok(Self::Pay),
            0xFF => Ok(Self::Custom),
            _ => Err(DecoderError::Custom("Unexpected action prefix")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Pay {
        receiver: Address,
        /// Transferred quantity.
        quantity: u64,
    },
    Custom {
        handler_id: u64,
        bytes: Bytes,
    },
}

impl Action {
    pub fn hash(&self) -> H256 {
        let rlp = self.rlp_bytes();
        Blake::blake(rlp)
    }

    pub fn tracker(&self) -> Option<Tracker> {
        Default::default()
    }

    pub fn verify(&self) -> Result<(), SyntaxError> {
        Ok(())
    }

    pub fn verify_with_params(&self, common_params: &CommonParams) -> Result<(), SyntaxError> {
        if let Some(network_id) = self.network_id() {
            let system_network_id = common_params.network_id();
            if network_id != system_network_id {
                return Err(SyntaxError::InvalidNetworkId(network_id))
            }
        }

        Ok(())
    }

    fn network_id(&self) -> Option<NetworkId> {
        None
    }
}

impl Encodable for Action {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            Action::Pay {
                receiver,
                quantity,
            } => {
                s.begin_list(3);
                s.append(&ActionTag::Pay);
                s.append(receiver);
                s.append(quantity);
            }
            Action::Custom {
                handler_id,
                bytes,
            } => {
                s.begin_list(3);
                s.append(&ActionTag::Custom);
                s.append(handler_id);
                s.append(bytes);
            }
        }
    }
}

impl Decodable for Action {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        match rlp.val_at(0)? {
            ActionTag::Pay => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 3,
                    })
                }
                Ok(Action::Pay {
                    receiver: rlp.val_at(1)?,
                    quantity: rlp.val_at(2)?,
                })
            }
            ActionTag::Custom => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpIncorrectListLen {
                        got: item_count,
                        expected: 3,
                    })
                }
                Ok(Action::Custom {
                    handler_id: rlp.val_at(1)?,
                    bytes: rlp.val_at(2)?,
                })
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn encode_and_decode_pay_action() {
        rlp_encode_and_decode_test!(Action::Pay {
            receiver: Address::random(),
            quantity: 300,
        });
    }
}
