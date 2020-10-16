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

pub mod state_machine;
mod state_manager;

use async_graphql::{InputValueError, InputValueResult, Scalar, ScalarType, Value as GqlValue};
use ckey::{verify, Ed25519Public as Public, Signature};
use primitives::H256;
use serde::{Deserialize, Serialize};
pub use state_manager::StateManager;

pub type TxSeq = u64;

pub fn assert_empty_arg(arg: &[u8]) -> Result<(), ()> {
    let a: std::collections::HashMap<String, String> = serde_cbor::from_slice(arg).map_err(|_| ())?;
    if a.is_empty() {
        Ok(())
    } else {
        Err(())
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SignedTransaction {
    pub signature: Signature,
    pub signer_public: Public,
    pub action: Vec<u8>,
}

impl SignedTransaction {
    pub fn verify(&self) -> Result<(), ()> {
        if verify(&self.signature, &self.action, &self.signer_public) {
            Ok(())
        } else {
            Err(())
        }
    }
}

pub struct GqlPublic(pub Public);

#[Scalar]
impl ScalarType for GqlPublic {
    fn parse(value: GqlValue) -> InputValueResult<Self> {
        if let GqlValue::String(s) = value {
            Ok(GqlPublic(
                Public::from_slice(
                    &hex::decode(&s).map_err(|_| InputValueError::Custom("Invalid public key".to_owned()))?,
                )
                .ok_or_else(|| InputValueError::Custom("Invalid public key".to_owned()))?,
            ))
        } else {
            Err(InputValueError::Custom("Invalid public key".to_owned()))
        }
    }

    fn to_value(&self) -> GqlValue {
        GqlValue::String(hex::encode(self.0.as_ref()))
    }
}

pub struct GqlH256(pub H256);

impl ScalarType for GqlH256 {
    fn parse(value: GqlValue) -> InputValueResult<Self> {
        if let GqlValue::String(s) = value {
            Ok(GqlH256(H256::from_slice(
                &hex::decode(&s).map_err(|_| InputValueError::Custom("Invalid public key".to_owned()))?,
            )))
        } else {
            Err(InputValueError::Custom("Invalid public key".to_owned()))
        }
    }

    fn to_value(&self) -> GqlValue {
        GqlValue::String(hex::encode(self.0.as_ref()))
    }
}

pub fn handle_gql_query<T: async_graphql::ObjectType + Send + Sync + 'static>(
    runtime: &tokio::runtime::Handle,
    root: T,
    query: &str,
    variables: &str,
) -> String {
    let variables = if let Ok(s) = (|| -> Result<_, ()> {
        Ok(async_graphql::Variables::parse_from_json(async_graphql::serde_json::from_str(variables).map_err(|_| ())?))
    })() {
        s
    } else {
        return "Failed to parse JSON".to_owned()
    };

    let schema = async_graphql::Schema::new(root, async_graphql::EmptyMutation, async_graphql::EmptySubscription);
    let query = async_graphql::QueryBuilder::new(query).variables(variables);
    let res = query.execute(&schema);
    async_graphql::serde_json::to_string(&async_graphql::http::GQLResponse(runtime.block_on(res))).unwrap()
}
