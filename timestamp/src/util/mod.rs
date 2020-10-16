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

use crate::common::*;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public, Signature};
use coordinator::module::*;
use foundry_module_rt::UserModule;
use remote_trait_object::raw_exchange::{HandleToExchange, Skeleton};
use remote_trait_object::Context as RtoContext;
use remote_trait_object::Service;

struct GraphQlRoot {}

pub struct GraphQlRequestHandler {
    /// A runtime to process the asynchronous result of the query
    tokio_runtime: tokio::runtime::Runtime,
}

#[async_graphql::Object]
impl GraphQlRoot {
    async fn sign_and_encode_tx(&self, private: String, content: String) -> async_graphql::FieldResult<String> {
        let private =
            Private::from_slice(&hex::decode(&private).map_err(|_| "Failed to parse private key".to_owned())?)
                .ok_or_else(|| "Invalid private key".to_owned())?;
        let content = hex::decode(&content).map_err(|_| "Failed to parse data".to_owned())?;
        let signature: Vec<u8> = ckey::sign(&content, &private).as_ref().to_vec();

        let tx = SignedTransaction {
            signature: Signature::from_slice(&signature).unwrap(),
            signer_public: private.public_key(),
            action: content,
        };
        Ok(hex::encode(serde_cbor::to_vec(&tx).unwrap()))
    }
}

impl GraphQlRequestHandler {
    pub(super) fn new() -> Self {
        Self {
            tokio_runtime: tokio::runtime::Builder::new().basic_scheduler().build().unwrap(),
        }
    }
}

impl Service for GraphQlRequestHandler {}

impl HandleGraphQlRequest for GraphQlRequestHandler {
    fn execute(&self, _session: SessionId, query: &str, variables: &str) -> String {
        handle_gql_query(self.tokio_runtime.handle(), GraphQlRoot {}, query, variables)
    }
}

pub struct Module {}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Self {}
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "handle-graphql-request" => {
                assert_empty_arg(ctor_arg).unwrap();
                Skeleton::new(Box::new(GraphQlRequestHandler::new()) as Box<dyn HandleGraphQlRequest>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(&mut self, _rto_context: &RtoContext, _name: &str, _handle: HandleToExchange) {
        panic!("Nothing to export!")
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
