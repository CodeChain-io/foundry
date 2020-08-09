// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::{Private, Public};
use primitives::H512;
use rlp::{Decodable, DecoderError, Encodable, Rlp, RlpStream};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
pub use sodiumoxide::crypto::sign::SIGNATUREBYTES;
use sodiumoxide::crypto::sign::{sign_detached, verify_detached, Signature};
use std::fmt;

pub fn sign(message: &[u8], private: &Private) -> Ed25519Signature {
    let Private(secret) = private;
    Ed25519Signature(sign_detached(message, secret))
}

pub fn verify(signature: &Ed25519Signature, message: &[u8], public: &Public) -> bool {
    let Public(public) = public;
    let Ed25519Signature(signature) = signature;
    verify_detached(signature, message, public)
}

#[derive(Clone, Copy, Hash, PartialEq, Eq)]
pub struct Ed25519Signature(Signature);

impl Ed25519Signature {
    pub fn random() -> Self {
        Self(Signature::from_slice(H512::random().as_ref()).unwrap())
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        Signature::from_slice(slice).map(Self)
    }
}

impl From<Signature> for Ed25519Signature {
    fn from(target: Signature) -> Self {
        Ed25519Signature(target)
    }
}

impl AsRef<[u8]> for Ed25519Signature {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl Default for Ed25519Signature {
    fn default() -> Self {
        Ed25519Signature(Signature([0; SIGNATUREBYTES]))
    }
}

impl Encodable for Ed25519Signature {
    fn rlp_append(&self, s: &mut RlpStream) {
        s.append_single_value(&self.as_ref());
    }
}

impl fmt::Debug for Ed25519Signature {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        self.0.fmt(formatter)
    }
}

impl Decodable for Ed25519Signature {
    fn decode(rlp: &Rlp<'_>) -> Result<Self, DecoderError> {
        let vec: Vec<u8> = rlp.as_val()?;
        let length = vec.len();
        if length == SIGNATUREBYTES {
            let mut array = [0; SIGNATUREBYTES];
            array.copy_from_slice(&vec[..SIGNATUREBYTES]);
            Ok(Ed25519Signature(Signature(array)))
        } else {
            Err(DecoderError::RlpInvalidLength {
                expected: SIGNATUREBYTES,
                got: length,
            })
        }
    }
}

impl Serialize for Ed25519Signature {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer, {
        let h512_pubkey = H512::from_slice(self.as_ref());
        h512_pubkey.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Ed25519Signature {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>, {
        let h512_signature = H512::deserialize(deserializer)?;
        Ok(Self::from_slice(h512_signature.as_ref()).expect("Bytes length was verified"))
    }
}

#[cfg(test)]
mod tests {
    use rlp::rlp_encode_and_decode_test;

    use super::*;

    #[test]
    fn signature_rlp() {
        rlp_encode_and_decode_test!(Ed25519Signature::random());
    }
}
