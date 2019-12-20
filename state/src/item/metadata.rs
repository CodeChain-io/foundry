// Copyright 2018-2019 Kodebox, Inc.
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

use ctypes::{CommonParams, TxHash};
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};

use crate::CacheableItem;

#[derive(Clone, Debug, Default, PartialEq)]
struct TermMetadata {
    last_term_finished_block_num: u64,
    current_term_id: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Metadata {
    hashes: Vec<TxHash>,
    term: TermMetadata,
    seq: u64,
    params: Option<CommonParams>,
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            hashes: vec![],
            term: Default::default(),
            seq: 0,
            params: None,
        }
    }

    pub fn seq(&self) -> u64 {
        self.seq
    }

    pub fn increase_seq(&mut self) {
        self.seq += 1;
    }

    pub fn params(&self) -> Option<&CommonParams> {
        self.params.as_ref()
    }

    pub fn set_params(&mut self, params: CommonParams) {
        self.params = Some(params);
    }

    pub fn increase_term_id(&mut self, last_term_finished_block_num: u64) {
        assert!(self.term.last_term_finished_block_num < last_term_finished_block_num);
        self.term.last_term_finished_block_num = last_term_finished_block_num;
        self.term.current_term_id += 1;
    }

    pub fn last_term_finished_block_num(&self) -> u64 {
        self.term.last_term_finished_block_num
    }

    pub fn current_term_id(&self) -> u64 {
        self.term.current_term_id
    }
}

impl Default for Metadata {
    fn default() -> Self {
        Self::new()
    }
}

impl CacheableItem for Metadata {
    type Address = MetadataAddress;

    // FIXME: I'm not sure of it.
    fn is_null(&self) -> bool {
        self.hashes.is_empty() && self.seq == 0
    }
}

const PREFIX: u8 = super::METADATA_PREFIX;

impl Encodable for Metadata {
    fn rlp_append(&self, s: &mut RlpStream) {
        const INITIAL_LEN: usize = 2;
        const TERM_LEN: usize = 2;
        const PARAMS_LEN: usize = 2;
        let mut len = INITIAL_LEN;

        let term_changed = self.term != Default::default();
        if term_changed {
            len += TERM_LEN;
        }

        let params_changed = self.seq != 0;
        if params_changed {
            if !term_changed {
                len += TERM_LEN;
            }
            len += PARAMS_LEN;
        }
        s.begin_list(len).append(&PREFIX).append_list(&self.hashes);
        if term_changed {
            s.append(&self.term.last_term_finished_block_num).append(&self.term.current_term_id);
        }
        if params_changed {
            if !term_changed {
                const DEFAULT_LAST_TERM_FINISHED_BLOCK_NUM: u64 = 0;
                const DEFAULT_CURRENT_TERM_ID: u64 = 0;
                s.append(&DEFAULT_LAST_TERM_FINISHED_BLOCK_NUM).append(&DEFAULT_CURRENT_TERM_ID);
            }
            s.append(&self.seq).append(self.params.as_ref().unwrap());
        }
    }
}

impl Decodable for Metadata {
    fn decode(rlp: &Rlp) -> Result<Self, DecoderError> {
        let (term, seq, params) = match rlp.item_count()? {
            2 => (TermMetadata::default(), 0, None),
            4 => (
                TermMetadata {
                    last_term_finished_block_num: rlp.val_at(2)?,
                    current_term_id: rlp.val_at(3)?,
                },
                0,
                None,
            ),
            6 => (
                TermMetadata {
                    last_term_finished_block_num: rlp.val_at(2)?,
                    current_term_id: rlp.val_at(3)?,
                },
                rlp.val_at(4)?,
                Some(rlp.val_at(5)?),
            ),
            item_count => {
                return Err(DecoderError::RlpInvalidLength {
                    got: item_count,
                    expected: 2,
                })
            }
        };
        let prefix = rlp.val_at::<u8>(0)?;
        if PREFIX != prefix {
            cdebug!(STATE, "{} is not an expected prefix for asset", prefix);
            return Err(DecoderError::Custom("Unexpected prefix"))
        }
        Ok(Self {
            hashes: rlp.list_at(1)?,
            term,
            seq,
            params,
        })
    }
}

#[derive(Clone, Copy, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct MetadataAddress(H256);

impl_address!(TOP, MetadataAddress, PREFIX);

impl MetadataAddress {
    pub fn new() -> Self {
        Self::from_transaction_hash(H256::from_slice(b"metadata address"), 0)
    }
}

#[cfg(test)]
mod tests {
    use ctypes::CommonParams;
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn parse_fail_return_none() {
        let hash = {
            let mut hash;
            loop {
                hash = H256::random();
                if hash[0] == PREFIX {
                    continue
                }
                break
            }
            hash
        };
        let address = MetadataAddress::from_hash(hash);
        assert!(address.is_none());
    }

    #[test]
    fn parse_return_some() {
        let hash = {
            let mut hash = H256::random();
            hash[0] = PREFIX;
            hash
        };
        let address = MetadataAddress::from_hash(hash);
        assert_eq!(Some(MetadataAddress(hash)), address);
    }

    #[test]
    fn metadata_with_0_seq() {
        let metadata = Metadata::default();
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn check_backward_compatibility() {
        let metadata = Metadata {
            hashes: vec![],
            term: Default::default(),
            seq: 0,
            params: None,
        };
        let mut rlp = RlpStream::new_list(2);
        rlp.append(&PREFIX).append_list::<H256, H256>(&[]);
        assert_eq!(metadata.rlp_bytes(), rlp.drain());
    }

    #[test]
    fn metadata_without_term_with_seq() {
        let metadata = Metadata {
            hashes: vec![],
            term: Default::default(),
            seq: 3,
            params: Some(CommonParams::default_for_test()),
        };
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn metadata_with_term_without_seq() {
        let metadata = Metadata {
            hashes: vec![],
            term: TermMetadata {
                last_term_finished_block_num: 1,
                current_term_id: 100,
            },
            seq: 0,
            params: None,
        };
        rlp_encode_and_decode_test!(metadata);
    }

    #[test]
    fn metadata_with_term_and_seq() {
        let metadata = Metadata {
            hashes: vec![],
            term: TermMetadata {
                last_term_finished_block_num: 1,
                current_term_id: 100,
            },
            seq: 3,
            params: Some(CommonParams::default_for_test()),
        };
        rlp_encode_and_decode_test!(metadata);
    }
}
