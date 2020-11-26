// Copyright 2020 Kodebox, Inc.
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

use primitives::H256;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sodiumoxide::crypto::box_::{gen_keypair, PublicKey};

#[derive(Clone, Eq, PartialEq, Debug)]
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
        Ok(Self::from_slice(h256_pubkey.as_ref()).expect("Bytes length was verified"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_default() {
        let _: Public = Default::default();
    }

    #[test]
    fn public_random() {
        let _: Public = Public::random();
    }

    #[test]
    fn serialiae_deserialize() {
        let random = Public::random();
        let json_string = serde_json::to_string(&random).unwrap();
        let deserialized: Public = serde_json::from_str(&json_string).unwrap();
        assert_eq!(random, deserialized);
    }
}
