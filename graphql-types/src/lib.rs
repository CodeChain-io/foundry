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

use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value as GqlValue};
use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use primitives::H256;

pub struct GqlPublic(pub Public);

#[Scalar]
impl ScalarType for GqlPublic {
    fn parse(value: GqlValue) -> InputValueResult<Self> {
        if let GqlValue::String(s) = value {
            Ok(GqlPublic(
                Public::from_slice(
                    &hex::decode(&s).map_err(|_| InputValueError::custom("Invalid public key".to_owned()))?,
                )
                .ok_or_else(|| InputValueError::custom("Invalid public key".to_owned()))?,
            ))
        } else {
            Err(InputValueError::custom("Invalid public key".to_owned()))
        }
    }

    fn to_value(&self) -> GqlValue {
        GqlValue::String(hex::encode(self.0.as_ref()))
    }
}

pub struct GqlPrivate(pub Private);

#[Scalar]
impl ScalarType for GqlPrivate {
    fn parse(value: GqlValue) -> InputValueResult<Self> {
        if let GqlValue::String(s) = value {
            Ok(GqlPrivate(
                Private::from_slice(
                    &hex::decode(&s).map_err(|_| InputValueError::custom("Invalid private key".to_owned()))?,
                )
                .ok_or_else(|| InputValueError::custom("Invalid private key".to_owned()))?,
            ))
        } else {
            Err(InputValueError::custom("Invalid private key".to_owned()))
        }
    }

    fn to_value(&self) -> GqlValue {
        GqlValue::String(hex::encode(self.0.as_ref()))
    }
}

pub struct GqlH256(pub H256);

#[Scalar]
impl ScalarType for GqlH256 {
    fn parse(value: GqlValue) -> InputValueResult<Self> {
        if let GqlValue::String(s) = value {
            Ok(GqlH256(H256::from_slice(
                &hex::decode(&s).map_err(|_| InputValueError::custom("Invalid public key".to_owned()))?,
            )))
        } else {
            Err(InputValueError::custom("Invalid public key".to_owned()))
        }
    }

    fn to_value(&self) -> GqlValue {
        GqlValue::String(hex::encode(self.0.as_ref()))
    }
}
