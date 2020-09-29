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

extern crate foundry_graphql as fgql;

mod common;

use actix_rt;
use actix_web::client::Client;
use actix_web::dev::Body;
use fgql::{GraphQlRequestHandler, ServerData};
use std::collections::HashMap;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;

fn graphql_handlers() -> HashMap<String, GraphQlRequestHandler> {
    (vec![("module1".to_owned(), GraphQlRequestHandler {
        session_needed: true,
        handler: Arc::from(common::create_handler()),
    })])
    .into_iter()
    .collect()
}

struct TestClient;

impl fgql::ManageSession for TestClient {
    fn new_session(&self, _block: ctypes::BlockId) -> coordinator::module::SessionId {
        123
    }

    fn end_session(&self, _session: coordinator::module::SessionId) {}
}

/// Creates the actual server.
///
/// If you want to use test utilities, try this instead.
/// ```
/// init_service(App::new().configure(|config: &mut ServiceConfig| app_configure(config, Arc::clone(&server_data))))
/// ```
fn create_server(port: u16) -> actix_web::dev::Server {
    let server_data = ServerData::new(Arc::new(TestClient), graphql_handlers());
    let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    fgql::run_server(server_data, socket).unwrap()
}

fn test_query() -> (HashMap<String, String>, String) {
    let graphql_query = r#"{account(name: "John"){balance}}"#.to_owned();
    let variables = "{}".to_owned();
    (
        (vec![("query".to_owned(), graphql_query), ("variables".to_owned(), variables)]).into_iter().collect(),
        r#"{"data":{"account":{"balance":10}}}"#.to_string(),
    )
}

fn test_query_variables() -> (HashMap<String, String>, String) {
    let graphql_query = r#"query Test($name: String){account(name: $name){balance}}"#.to_owned();
    let variables = r#"{"name": "John"}"#.to_owned();
    (
        (vec![("query".to_owned(), graphql_query), ("variables".to_owned(), variables)]).into_iter().collect(),
        r#"{"data":{"account":{"balance":10}}}"#.to_owned(),
    )
}

#[actix_rt::test]
async fn run_server() {
    let port = 4000;
    create_server(port);
}

#[actix_rt::test]
async fn request_get() {
    let port = 4001;
    let _server = create_server(port);
    let client = Client::new();
    let (query, expected) = test_query();

    let request = client.get(&format!("http://localhost:{}/module1/graphql", port)).query(&query).unwrap();
    let response_bytes = request.send().await.unwrap().body().await.unwrap();
    let response = std::str::from_utf8(&response_bytes).expect("GraphQL server must return utf8-encoded string");
    assert_eq!(response, expected);
}

#[actix_rt::test]
async fn request_get_with_variables() {
    let port = 4002;
    let _server = create_server(port);
    let client = Client::new();
    let (query, expected) = test_query_variables();

    let request = client.get(&format!("http://localhost:{}/module1/graphql", port)).query(&query).unwrap();
    let response_bytes = request.send().await.unwrap().body().await.unwrap();
    let response = std::str::from_utf8(&response_bytes).expect("GraphQL server must return utf8-encoded string");
    assert_eq!(response, expected);
}

#[actix_rt::test]
async fn request_post() {
    let port = 4003;
    let _server = create_server(port);
    let client = Client::new();
    let (query, expected) = test_query();
    let body = Body::Bytes(serde_json::to_vec(&query).unwrap().into());

    let request =
        client.post(&format!("http://localhost:{}/module1/graphql", port)).header("content-type", "application/json");
    let response_bytes = request.send_body(body).await.unwrap().body().await.unwrap();
    let response = std::str::from_utf8(&response_bytes).expect("GraphQL server must return utf8-encoded string");
    assert_eq!(response, expected);
}

#[actix_rt::test]
async fn request_post_with_variables() {
    let port = 4004;
    let _server = create_server(port);
    let client = Client::new();
    let (query, expected) = test_query_variables();
    let body = Body::Bytes(serde_json::to_vec(&query).unwrap().into());

    let request =
        client.post(&format!("http://localhost:{}/module1/graphql", port)).header("content-type", "application/json");
    let response_bytes = request.send_body(body).await.unwrap().body().await.unwrap();
    let response = std::str::from_utf8(&response_bytes).expect("GraphQL server must return utf8-encoded string");
    assert_eq!(response, expected);
}
