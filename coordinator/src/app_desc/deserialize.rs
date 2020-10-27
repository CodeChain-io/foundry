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
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use primitives::H256;
use serde::Deserialize;
use serde::{de::Error, Deserializer};

pub trait TryFromBytes: Sized {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, String>;
}

#[derive(Debug)]
pub struct Hex<T: TryFromBytes> {
    pub value: T,
}

impl<'de, T: TryFromBytes> Deserialize<'de> for Hex<T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>, {
        let bytes: Vec<u8> = hex::deserialize(deserializer)?;
        let deserialized = T::try_from_bytes(&bytes).map_err(Error::custom)?;
        Ok(Hex {
            value: deserialized,
        })
    }
}

impl TryFromBytes for H256 {
    fn try_from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() != 32 {
            return Err(format!("invalie length {}", bytes.len()))
        }
        Ok(H256::from_slice(bytes))
    }
}
