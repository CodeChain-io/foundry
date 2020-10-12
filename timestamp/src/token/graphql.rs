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

use super::state_machine::GetAccount;
use super::types::*;
use super::{ServiceHandler, StateMachine};
use crate::common::*;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::module::*;
use remote_trait_object::Service;
use std::sync::Arc;

struct GraphQlRoot {
    state_machine: StateMachine,
}

#[async_graphql::Object]
impl GraphQlRoot {
    async fn account(&self, public: GqlPublic) -> Option<Account> {
        self.state_machine
            .execute_access(GetAccount {
                public: &public.0,
                default: false,
            })
            .ok()
    }
}

#[async_graphql::Object]
impl Token {
    async fn issuer(&self) -> String {
        hex::encode(self.issuer.as_ref())
    }
}

#[async_graphql::Object]
impl Account {
    async fn tokens(&self) -> &Vec<Token> {
        &self.tokens
    }
}

pub struct GraphQlRequestHandler {
    service_handler: Arc<ServiceHandler>,

    /// A runtime to process the asynchronous result of the query
    tokio_runtime: tokio::runtime::Runtime,
}

impl GraphQlRequestHandler {
    pub(super) fn new(service_handler: Arc<ServiceHandler>) -> Self {
        Self {
            service_handler,
            tokio_runtime: tokio::runtime::Builder::new().basic_scheduler().build().unwrap(),
        }
    }
}

impl Service for GraphQlRequestHandler {}

impl HandleGraphQlRequest for GraphQlRequestHandler {
    fn execute(&self, session: SessionId, query: &str, variables: &str) -> String {
        handle_gql_query(
            self.tokio_runtime.handle(),
            GraphQlRoot {
                state_machine: self.service_handler.create_state_machine(session),
            },
            query,
            variables,
        )
    }
}
