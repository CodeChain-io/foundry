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
use ccrypto::blake256;
use ckey::Ed25519Public as Public;
use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq)]
pub struct CompactValidatorEntry {
    pub public_key: Public,
    pub delegation: u64,
}

// It will be hashed in the header.
#[derive(Serialize, Deserialize, Clone, Debug, Default, Eq, PartialEq)]
pub struct CompactValidatorSet(Vec<CompactValidatorEntry>);
impl CompactValidatorSet {
    pub fn new(x: Vec<CompactValidatorEntry>) -> Self {
        Self(x)
    }

    pub fn hash(&self) -> H256 {
        blake256(self.rlp_bytes())
    }
}

impl Deref for CompactValidatorSet {
    type Target = Vec<CompactValidatorEntry>;
    fn deref(&self) -> &Vec<CompactValidatorEntry> {
        &self.0
    }
}

impl DerefMut for CompactValidatorSet {
    fn deref_mut(&mut self) -> &mut Vec<CompactValidatorEntry> {
        &mut self.0
    }
}

impl From<CompactValidatorSet> for Vec<CompactValidatorEntry> {
    fn from(compact_validator_set: CompactValidatorSet) -> Self {
        compact_validator_set.0
    }
}

impl Encodable for CompactValidatorSet {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.begin_list(self.0.len() * 2);
        for validator in self.0.iter() {
            s.append(&validator.public_key);
            s.append(&validator.delegation);
        }
    }
}

impl Decodable for CompactValidatorSet {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let item_count = rlp.item_count()?;
        if item_count % 2 == 1 {
            return Err(DecoderError::RlpInvalidLength {
                expected: item_count + 1,
                got: item_count,
            })
        }
        let mut vec = Vec::with_capacity(item_count / 2);
        // TODO: Optimzie the below code
        for i in 0..(item_count / 2) {
            vec.push(CompactValidatorEntry {
                public_key: rlp.val_at(i * 2)?,
                delegation: rlp.val_at(i * 2 + 1)?,
            });
        }
        Ok(Self::new(vec))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{rngs::StdRng, Rng};
    use rlp::rlp_encode_and_decode_test;

    #[test]
    fn encode_and_decode() {
        let iteration = 100;

        let seed = [0 as u8; 32];
        let mut rng: StdRng = rand::SeedableRng::from_seed(seed);

        for _ in 0..iteration {
            let mut vset = CompactValidatorSet::new(Vec::new());
            let n = rng.gen::<u8>();

            for _ in 0..n {
                vset.0.push(CompactValidatorEntry {
                    public_key: Public::random(),
                    delegation: rng.gen::<u64>(),
                })
            }
            rlp_encode_and_decode_test!(vset);
        }
    }
}
