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

use super::public::Public;
use primitives::H512;
use sodiumoxide::crypto::sign::{gen_keypair, SecretKey, SECRETKEYBYTES};
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq)]
// The inner type SecretKey clears its memory when it is dropped
pub struct Private(pub(crate) SecretKey);

impl Private {
    pub fn random() -> Self {
        let (_, secret) = gen_keypair();
        Private(secret)
    }

    pub fn public_key(&self) -> Public {
        self.0.public_key().into()
    }

    pub fn from_slice(slice: &[u8]) -> Option<Self> {
        if slice == &[0; SECRETKEYBYTES][..] {
            None
        } else {
            SecretKey::from_slice(slice).map(Self)
        }
    }
}

impl FromStr for Private {
    type Err = crate::Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let for_slice = H512::from_str(s).map_err(|e| crate::Error::Custom(format!("{:?}", e)))?;
        Ok(SecretKey::from_slice(&for_slice).expect("H512 has length 64").into())
    }
}

impl AsRef<[u8]> for Private {
    fn as_ref(&self) -> &[u8] {
        self.0.as_ref()
    }
}

impl From<SecretKey> for Private {
    fn from(k: SecretKey) -> Self {
        Private(k)
    }
}
