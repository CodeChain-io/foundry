// Copyright 2019-2020 Kodebox, Inc.
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

use crate::ibc::channel_04::types::ChannelOrder;
use crate::ibc::channel_04::types::Packet;
use primitives::Bytes;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DatagramTag {
    CreateClient = 1,
    UpdateClient = 2,
    ConnOpenInit = 3,
    ConnOpenTry = 4,
    ConnOpenAck = 5,
    ConnOpenConfirm = 6,
    ChanOpenInit = 7,
    ChanOpenTry = 8,
    ChanOpenAck = 9,
    ChanOpenConfirm = 10,
    ChanCloseInit = 11,
    ChanCloseConfirm = 12,
    SendPacket = 13,
    RecvPacket = 14,
    AcknowledgePacket = 15,
}

impl Encodable for DatagramTag {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&(*self as u8));
    }
}

impl Decodable for DatagramTag {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let byte: u8 = rlp.as_val()?;
        match byte {
            1 => Ok(DatagramTag::CreateClient),
            2 => Ok(DatagramTag::UpdateClient),
            3 => Ok(DatagramTag::ConnOpenInit),
            4 => Ok(DatagramTag::ConnOpenTry),
            5 => Ok(DatagramTag::ConnOpenAck),
            6 => Ok(DatagramTag::ConnOpenConfirm),
            7 => Ok(DatagramTag::ChanOpenInit),
            8 => Ok(DatagramTag::ChanOpenTry),
            9 => Ok(DatagramTag::ChanOpenAck),
            10 => Ok(DatagramTag::ChanOpenConfirm),
            11 => Ok(DatagramTag::ChanCloseInit),
            12 => Ok(DatagramTag::ChanCloseConfirm),
            13 => Ok(DatagramTag::SendPacket),
            14 => Ok(DatagramTag::RecvPacket),
            15 => Ok(DatagramTag::AcknowledgePacket),
            _ => Err(DecoderError::Custom("Unexpected DatagramTag Value")),
        }
    }
}

// internal struct representation: this detour will reduce meaningless code of RLP en/decoding
// This is because of RLP macro's weak support.
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ChanOpenInit {
    pub tag: DatagramTag,
    pub order: ChannelOrder,
    pub connection: String,
    pub channel_identifier: String,
    pub counterparty_channel_identifier: String,
    pub version: String,
}
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ChanOpenTry {
    pub tag: DatagramTag,
    pub order: ChannelOrder,
    pub connection: String,
    pub channel_identifier: String,
    pub counterparty_channel_identifier: String,
    pub version: String,
    pub counterparty_version: String,
    pub proof_init: Bytes,
    pub proof_height: u64,
}
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ChanOpenAck {
    pub tag: DatagramTag,
    pub channel_identifier: String,
    pub counterparty_version: String,
    pub proof_try: Bytes,
    pub proof_height: u64,
}
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ChanOpenConfirm {
    pub tag: DatagramTag,
    pub channel_identifier: String,
    pub proof_ack: Bytes,
    pub proof_height: u64,
}
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ChanCloseInit {
    pub tag: DatagramTag,
    pub channel_identifier: String,
}
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct ChanCloseConfirm {
    pub tag: DatagramTag,
    pub channel_identifier: String,
    pub proof_init: Bytes,
    pub proof_height: u64,
}
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct SendPacket {
    pub tag: DatagramTag,
    pub packet: Packet,
}
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct RecvPacket {
    pub tag: DatagramTag,
    pub packet: Packet,
    pub proof: Bytes,
    pub proof_height: u64,
    pub ack: Bytes,
}
#[derive(RlpEncodable, RlpDecodable, PartialEq, Debug)]
pub struct AcknowledgePacket {
    pub tag: DatagramTag,
    pub packet: Packet,
    pub ack: Bytes,
    pub proof: Bytes,
    pub proof_height: u64,
}

#[derive(Debug, PartialEq)]
pub enum Datagram {
    CreateClient {
        id: String,
        kind: u8,
        consensus_state: Bytes,
        data: Bytes,
    },
    UpdateClient {
        id: String,
        header: Bytes,
    },
    ConnOpenInit {
        identifier: String,
        desired_counterparty_connection_identifier: String,
        counterparty_prefix: String,
        client_identifier: String,
        counterparty_client_identifier: String,
    },
    ConnOpenTry {
        desired_identifier: String,
        counterparty_connection_identifier: String,
        counterparty_prefix: String,
        counterparty_client_identifier: String,
        client_identifier: String,
        proof_init: Bytes,
        proof_consensus: Bytes,
        proof_height: u64,
        consensus_height: u64,
    },
    ConnOpenAck {
        identifier: String,
        proof_try: Bytes,
        proof_consensus: Bytes,
        proof_height: u64,
        consensus_height: u64,
    },
    ConnOpenConfirm {
        identifier: String,
        proof_ack: Bytes,
        proof_height: u64,
    },
    ChanOpenInit {
        raw: ChanOpenInit,
    },
    ChanOpenTry {
        raw: ChanOpenTry,
    },
    ChanOpenAck {
        raw: ChanOpenAck,
    },
    ChanOpenConfirm {
        raw: ChanOpenConfirm,
    },
    ChanCloseInit {
        raw: ChanCloseInit,
    },
    ChanCloseConfirm {
        raw: ChanCloseConfirm,
    },
    SendPacket {
        raw: SendPacket,
    },
    RecvPacket {
        raw: RecvPacket,
    },
    AcknowledgePacket {
        raw: AcknowledgePacket,
    },
}

impl Encodable for Datagram {
    fn rlp_append(&self, s: &mut RlpStream) {
        match self {
            Datagram::CreateClient {
                id,
                kind,
                consensus_state,
                data,
            } => {
                s.begin_list(5)
                    .append(&DatagramTag::CreateClient)
                    .append(id)
                    .append(kind)
                    .append(consensus_state)
                    .append(data);
            }
            Datagram::UpdateClient {
                id,
                header,
            } => {
                s.begin_list(3).append(&DatagramTag::UpdateClient).append(id).append(header);
            }
            Datagram::ConnOpenInit {
                identifier,
                desired_counterparty_connection_identifier,
                counterparty_prefix,
                client_identifier,
                counterparty_client_identifier,
            } => {
                s.begin_list(6);
                s.append(&DatagramTag::ConnOpenInit)
                    .append(identifier)
                    .append(desired_counterparty_connection_identifier)
                    .append(counterparty_prefix)
                    .append(client_identifier)
                    .append(counterparty_client_identifier);
            }
            Datagram::ConnOpenTry {
                desired_identifier,
                counterparty_connection_identifier,
                counterparty_prefix,
                counterparty_client_identifier,
                client_identifier,
                proof_init,
                proof_consensus,
                proof_height,
                consensus_height,
            } => {
                s.begin_list(10);
                s.append(&DatagramTag::ConnOpenTry)
                    .append(desired_identifier)
                    .append(counterparty_connection_identifier)
                    .append(counterparty_prefix)
                    .append(counterparty_client_identifier)
                    .append(client_identifier)
                    .append(proof_init)
                    .append(proof_consensus)
                    .append(proof_height)
                    .append(consensus_height);
            }
            Datagram::ConnOpenAck {
                identifier,
                proof_try,
                proof_consensus,
                proof_height,
                consensus_height,
            } => {
                s.begin_list(6);
                s.append(&DatagramTag::ConnOpenAck)
                    .append(identifier)
                    .append(proof_try)
                    .append(proof_consensus)
                    .append(proof_height)
                    .append(consensus_height);
            }
            Datagram::ConnOpenConfirm {
                identifier,
                proof_ack,
                proof_height,
            } => {
                s.begin_list(4);
                s.append(&DatagramTag::ConnOpenConfirm).append(identifier).append(proof_ack).append(proof_height);
            }
            Datagram::ChanOpenInit {
                raw,
            } => {
                s.append_single_value(raw);
            }
            Datagram::ChanOpenTry {
                raw,
            } => {
                s.append_single_value(raw);
            }
            Datagram::ChanOpenAck {
                raw,
            } => {
                s.append_single_value(raw);
            }
            Datagram::ChanOpenConfirm {
                raw,
            } => {
                s.append_single_value(raw);
            }
            Datagram::ChanCloseInit {
                raw,
            } => {
                s.append_single_value(raw);
            }
            Datagram::ChanCloseConfirm {
                raw,
            } => {
                s.append_single_value(raw);
            }
            Datagram::SendPacket {
                raw,
            } => {
                s.append_single_value(raw);
            }
            Datagram::RecvPacket {
                raw,
            } => {
                s.append_single_value(raw);
            }
            Datagram::AcknowledgePacket {
                raw,
            } => {
                s.append_single_value(raw);
            }
        };
    }
}

impl Decodable for Datagram {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let tag = rlp.val_at(0)?;
        match tag {
            DatagramTag::CreateClient => {
                let item_count = rlp.item_count()?;
                if item_count != 5 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 5,
                        got: item_count,
                    })
                }
                Ok(Datagram::CreateClient {
                    id: rlp.val_at(1)?,
                    kind: rlp.val_at(2)?,
                    consensus_state: rlp.val_at(3)?,
                    data: rlp.val_at(4)?,
                })
            }
            DatagramTag::UpdateClient => {
                let item_count = rlp.item_count()?;
                if item_count != 3 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 3,
                        got: item_count,
                    })
                }
                Ok(Datagram::UpdateClient {
                    id: rlp.val_at(1)?,
                    header: rlp.val_at(2)?,
                })
            }
            DatagramTag::ConnOpenInit => {
                let item_count = rlp.item_count()?;
                if item_count != 6 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 6,
                        got: item_count,
                    })
                }
                Ok(Datagram::ConnOpenInit {
                    identifier: rlp.val_at(1)?,
                    desired_counterparty_connection_identifier: rlp.val_at(2)?,
                    counterparty_prefix: rlp.val_at(3)?,
                    client_identifier: rlp.val_at(4)?,
                    counterparty_client_identifier: rlp.val_at(5)?,
                })
            }
            DatagramTag::ConnOpenTry => {
                let item_count = rlp.item_count()?;
                if item_count != 10 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 10,
                        got: item_count,
                    })
                }
                Ok(Datagram::ConnOpenTry {
                    desired_identifier: rlp.val_at(1)?,
                    counterparty_connection_identifier: rlp.val_at(2)?,
                    counterparty_prefix: rlp.val_at(3)?,
                    counterparty_client_identifier: rlp.val_at(4)?,
                    client_identifier: rlp.val_at(5)?,
                    proof_init: rlp.val_at(6)?,
                    proof_consensus: rlp.val_at(7)?,
                    proof_height: rlp.val_at(8)?,
                    consensus_height: rlp.val_at(9)?,
                })
            }
            DatagramTag::ConnOpenAck => {
                let item_count = rlp.item_count()?;
                if item_count != 6 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 6,
                        got: item_count,
                    })
                }
                Ok(Datagram::ConnOpenAck {
                    identifier: rlp.val_at(1)?,
                    proof_try: rlp.val_at(2)?,
                    proof_consensus: rlp.val_at(3)?,
                    proof_height: rlp.val_at(4)?,
                    consensus_height: rlp.val_at(5)?,
                })
            }
            DatagramTag::ConnOpenConfirm => {
                let item_count = rlp.item_count()?;
                if item_count != 4 {
                    return Err(DecoderError::RlpInvalidLength {
                        expected: 4,
                        got: item_count,
                    })
                }
                Ok(Datagram::ConnOpenConfirm {
                    identifier: rlp.val_at(1)?,
                    proof_ack: rlp.val_at(2)?,
                    proof_height: rlp.val_at(3)?,
                })
            }
            DatagramTag::ChanOpenInit => Ok(Datagram::ChanOpenInit {
                raw: rlp.as_val()?,
            }),
            DatagramTag::ChanOpenTry => Ok(Datagram::ChanOpenTry {
                raw: rlp.as_val()?,
            }),
            DatagramTag::ChanOpenAck => Ok(Datagram::ChanOpenAck {
                raw: rlp.as_val()?,
            }),
            DatagramTag::ChanOpenConfirm => Ok(Datagram::ChanOpenConfirm {
                raw: rlp.as_val()?,
            }),
            DatagramTag::ChanCloseInit => Ok(Datagram::ChanCloseInit {
                raw: rlp.as_val()?,
            }),
            DatagramTag::ChanCloseConfirm => Ok(Datagram::ChanCloseConfirm {
                raw: rlp.as_val()?,
            }),
            DatagramTag::SendPacket => Ok(Datagram::SendPacket {
                raw: rlp.as_val()?,
            }),
            DatagramTag::RecvPacket => Ok(Datagram::RecvPacket {
                raw: rlp.as_val()?,
            }),
            DatagramTag::AcknowledgePacket => Ok(Datagram::AcknowledgePacket {
                raw: rlp.as_val()?,
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ibc::channel_04::types::Sequence;
    use rlp::{self, rlp_encode_and_decode_test};

    #[test]
    fn conn_open_init() {
        let conn_open_init = Datagram::ConnOpenInit {
            identifier: "identifier".to_owned(),
            desired_counterparty_connection_identifier: "desired_counterparty_connection_identifier".to_owned(),
            counterparty_prefix: "counterparty_prefix".to_owned(),
            client_identifier: "client_identifier".to_owned(),
            counterparty_client_identifier: "counterparty_client_identifier".to_owned(),
        };
        rlp_encode_and_decode_test!(conn_open_init);
    }

    #[test]
    fn conn_open_try() {
        let conn_open_try = Datagram::ConnOpenTry {
            desired_identifier: "desired_identifier".to_owned(),
            counterparty_connection_identifier: "counterparty_connection_identifier".to_owned(),
            counterparty_prefix: "counterparty_prefix".to_owned(),
            counterparty_client_identifier: "counterparty_client_identifier".to_owned(),
            client_identifier: "client_identifier".to_owned(),
            proof_init: b"proof_init".to_vec(),
            proof_consensus: b"proof_consensus".to_vec(),
            proof_height: 1,
            consensus_height: 2,
        };
        rlp_encode_and_decode_test!(conn_open_try);
    }

    #[test]
    fn conn_open_ack() {
        let conn_open_ack = Datagram::ConnOpenAck {
            identifier: "identifier".to_owned(),
            proof_try: b"proof_try".to_vec(),
            proof_consensus: b"proof_consensus".to_vec(),
            proof_height: 1,
            consensus_height: 2,
        };
        rlp_encode_and_decode_test!(conn_open_ack);
    }

    #[test]
    fn conn_open_confirm() {
        let conn_open_confirm = Datagram::ConnOpenConfirm {
            identifier: "identifier".to_owned(),
            proof_ack: b"proof_ack".to_vec(),
            proof_height: 1,
        };
        rlp_encode_and_decode_test!(conn_open_confirm);
    }

    #[test]
    fn chann_open_init() {
        let chan_open_init = Datagram::ChanOpenInit {
            raw: ChanOpenInit {
                tag: DatagramTag::ChanOpenInit,
                order: ChannelOrder::ORDERED,
                connection: "connection".to_owned(),
                channel_identifier: "channel".to_owned(),
                counterparty_channel_identifier: "counterparty_channel".to_owned(),
                version: "version".to_owned(),
            },
        };
        rlp_encode_and_decode_test!(chan_open_init);
    }

    #[test]
    fn send_packet() {
        let send_packet = Datagram::SendPacket {
            raw: SendPacket {
                tag: DatagramTag::SendPacket,
                packet: Packet {
                    sequence: Sequence {
                        raw: 1,
                    },
                    timeout_height: 32,
                    source_port: "source_port".to_owned(),
                    source_channel: "source_channel".to_owned(),
                    dest_port: "dest_port".to_owned(),
                    dest_channel: "dest_channel".to_owned(),
                    data: b"data".to_vec(),
                },
            },
        };
        rlp_encode_and_decode_test!(send_packet);
    }
}
