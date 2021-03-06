// Copyright 2018, 2020 Kodebox, Inc.
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

#[macro_use]
extern crate clogger;
#[macro_use]
extern crate log;

mod block;
mod header;

use block::Block;
use ccore::Client;
use ccore::{BlockChainClient, BlockChainTrait};
use coordinator::module::{HandleGraphQlRequest, SessionId};
use foundry_graphql_types::*;
use remote_trait_object::Service;
use std::sync::Arc;

#[derive(Clone)]
struct QueryRoot {
    client: Arc<Client>,
}

#[async_graphql::Object]
impl QueryRoot {
    async fn block(&self, number: Option<u64>) -> Option<Block> {
        let id = match number {
            Some(n) => ctypes::BlockId::Number(n),
            None => ctypes::BlockId::Latest,
        };
        self.client.block(&id).map(|x| Block::new(x.rlp().as_raw().to_vec()))
    }

    async fn event(&self, tx_hash: GqlH256) -> async_graphql::Result<Vec<GqlBytes>> {
        Ok(self
            .client
            .events_by_tx_hash(&ctypes::TxHash::from(tx_hash.0))
            .into_iter()
            .map(|x| GqlBytes(x.value))
            .collect())
    }

    /// FIXME: Design a general query scheme to handle both block to tx and tx to block.
    async fn transaction(&self, tx_hash: GqlH256) -> Option<u64> {
        self.client.transaction(&ctypes::TxHash::from(tx_hash.0).into()).map(|tx| tx.block_number)
    }
}

#[derive(Clone)]
struct MutationRoot {
    client: Arc<Client>,
}

#[async_graphql::Object]
impl MutationRoot {
    async fn send_transaction(&self, tx_type: String, body: GqlBytes) -> async_graphql::Result<String> {
        let tx = coordinator::Transaction::new(tx_type, body.0);
        // NOTE: Check `queue_own_transaction()` won't cause a deadlock, especially when called by the async runtime.
        Ok(match self.client.queue_own_transaction(tx) {
            Ok(_) => "Done".to_owned(),
            Err(_) => "Failed".to_owned(),
        })
    }
}

pub struct EngineLevelGraphQlHandler {
    query_root: QueryRoot,
    mutation_root: MutationRoot,

    tokio_runtime: tokio::runtime::Runtime,
}

impl EngineLevelGraphQlHandler {
    pub fn new(client: Arc<Client>) -> Self {
        Self {
            query_root: QueryRoot {
                client: Arc::clone(&client),
            },
            mutation_root: MutationRoot {
                client,
            },
            tokio_runtime: tokio::runtime::Builder::new().basic_scheduler().build().unwrap(),
        }
    }
}

impl Service for EngineLevelGraphQlHandler {}

impl HandleGraphQlRequest for EngineLevelGraphQlHandler {
    fn execute(&self, _session_id: SessionId, query: &str, variables: &str) -> String {
        let variables = if let Ok(s) = (|| -> Result<_, ()> {
            Ok(async_graphql::Variables::from_json(serde_json::from_str(variables).map_err(|_| ())?))
        })() {
            s
        } else {
            return "Failed to parse JSON".to_owned()
        };

        let schema = async_graphql::Schema::new(
            self.query_root.clone(),
            self.mutation_root.clone(),
            async_graphql::EmptySubscription,
        );
        let query = async_graphql::Request::new(query).variables(variables);
        cdebug!(GRAPHQL, "Request {:?}", query);
        let res = schema.execute(query);

        // FIXME: We can't use tokio runtime inside another tokio. We spawn a new thread to avoid such restriciton.
        let response = crossbeam::thread::scope(|s| {
            let j = s.spawn(|_| serde_json::to_string(&self.tokio_runtime.handle().block_on(res)).unwrap());
            j.join().unwrap()
        })
        .unwrap();

        cdebug!(GRAPHQL, "response {:?}", response);
        response
    }
}
