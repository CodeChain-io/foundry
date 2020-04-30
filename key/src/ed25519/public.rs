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

use primitives::H256;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sodiumoxide::crypto::sign::{gen_keypair, PublicKey, PUBLICKEYBYTES};
use std::str::FromStr;

#[derive(Debug, Copy, Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct Public(pub(crate) PublicKey);

impl Default for Public {
    fn default() -> Self {
        const ZERO: [u8; 32] = [0; 32];
        Self::from_slice(&ZERO).unwrap()
    }
}

impl Public {
    pub fn random() -> Self {
        let (public, _) = gen_keypair();
        public.into()
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        PublicKey::from_slice(slice).map(Self)
    }

    pub fn is_zero(&self) -> bool {
        self.as_ref() == [0; PUBLICKEYBYTES]
    }
}

impl From<u64> for Public {
    fn from(integer: u64) -> Self {
        let for_slice: H256 = integer.into();
        PublicKey::from_slice(&for_slice).unwrap().into()
    }
}

impl FromStr for Public {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let for_slice = H256::from_str(s).map_err(|_| crate::Error::InvalidPublic(s.to_string()))?;
        Ok(PublicKey::from_slice(&for_slice).expect("H256 has length 32").into())
    }
}

impl AsRef<[u8]> for Public {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<PublicKey> for Public {
    fn from(k: PublicKey) -> Self {
        Public(k)
    }
}

impl Encodable for Public {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&self.as_ref());
    }
}

impl Decodable for Public {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let vec: Vec<u8> = rlp.as_val()?;
        let length = vec.len();
        if length == PUBLICKEYBYTES {
            let mut array = [0; PUBLICKEYBYTES];
            array.copy_from_slice(&vec[..PUBLICKEYBYTES]);
            Ok(Public(PublicKey(array)))
        } else {
            Err(DecoderError::RlpInvalidLength {
                expected: PUBLICKEYBYTES,
                got: length,
            })
        }
    }
}

impl Serialize for Public {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
        let h256_pubkey = H256::from_slice(self.as_ref());
        h256_pubkey.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Public {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>, {
        let h256_pubkey = H256::deserialize(deserializer)?;
        Ok(Self::from_slice(&h256_pubkey).expect("Bytes length was verified"))
    }
}

#[cfg(test)]
mod tests {
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn public_key_rlp() {
        rlp_encode_and_decode_test!(Public::random());
    }
}
