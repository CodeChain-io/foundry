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

mod handler;

use actix_web::{
    dev::Server,
    error::{ErrorBadRequest, ErrorMethodNotAllowed, ErrorNotFound, ErrorUnsupportedMediaType},
    http::{header::CONTENT_TYPE, Method},
    web,
    web::{Payload, Query, ServiceConfig},
    App, FromRequest, HttpRequest, HttpResponse, HttpServer, Result,
};
use coordinator::module::{HandleGraphQlRequest, SessionId};
use futures::StreamExt;
pub use handler::handle_gql_query;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;

// TODO: replace with the real Client.
pub struct Client {}

impl Client {
    fn allocate_session(&self, _height: Option<u64>) -> SessionId {
        123
    }
}

pub struct GraphQlRequestHandler {
    pub handler: Arc<dyn HandleGraphQlRequest>,
    pub session_needed: bool,
}

pub struct ServerData {
    client: Arc<Client>,
    /// Name to (session_needed, handler)
    graphql_handlers: HashMap<String, GraphQlRequestHandler>,
}

impl ServerData {
    pub fn new(client: Arc<Client>, graphql_handlers: HashMap<String, GraphQlRequestHandler>) -> Self {
        Self {
            client,
            graphql_handlers,
        }
    }
}

async fn handle_request(
    request: HttpRequest,
    payload: Payload,
    path: web::Path<String>,
    server_data: web::Data<Arc<ServerData>>,
) -> Result<HttpResponse> {
    let module_name = path.into_inner();
    if let Some(GraphQlRequestHandler {
        session_needed,
        handler,
    }) = server_data.graphql_handlers.get(&module_name)
    {
        let session = if *session_needed {
            let height = None;
            server_data.client.allocate_session(height)
        } else {
            0
        };

        match *request.method() {
            Method::POST => handle_post(request, payload, &**handler, session).await,
            Method::GET => handle_get(request, payload, &**handler, session).await,
            _ => Err(ErrorMethodNotAllowed("GraphQL requests can only be sent with GET or POST")),
        }
    } else {
        Err(ErrorNotFound(format!("Module not found: {}", module_name)))
    }
}

const MAX_SIZE: usize = 262_144; // max payload size is 256k

async fn handle_post(
    request: HttpRequest,
    mut payload: Payload,
    handler: &dyn HandleGraphQlRequest,
    session: SessionId,
) -> Result<HttpResponse> {
    let content_type_header = request.headers().get(CONTENT_TYPE).and_then(|hv| hv.to_str().ok());
    let query: HashMap<String, String> = match content_type_header {
        Some("application/json") | Some("application/graphql") => {
            let mut body = web::BytesMut::new();
            while let Some(chunk) = payload.next().await {
                let chunk = chunk?;
                // limit max size of in-memory payload
                if (body.len() + chunk.len()) > MAX_SIZE {
                    return Err(ErrorBadRequest("overflow"))
                }
                body.extend_from_slice(&chunk);
            }
            Ok(serde_json::from_slice(body.as_ref()).unwrap())
        }
        _ => Err(ErrorUnsupportedMediaType(
            "GraphQL requests should have content type `application/json` or `application/graphql`",
        )),
    }?;
    let graphql_query = query.get("query").ok_or_else(HttpResponse::BadRequest)?;
    let variables = query.get("variables").ok_or_else(HttpResponse::BadRequest)?;

    let graphql_response = handler.execute(session, graphql_query, variables);
    Ok(HttpResponse::Ok().content_type("application/json").body(graphql_response))
}

async fn handle_get(
    request: HttpRequest,
    payload: Payload,
    handler: &dyn HandleGraphQlRequest,
    session: SessionId,
) -> Result<HttpResponse> {
    let mut payload = payload.into_inner();
    let query: Query<HashMap<String, String>> =
        Query::from_request(&request, &mut payload).await.map_err(|_| HttpResponse::BadRequest())?;
    let query = query.into_inner();
    let graphql_query = query.get("query").ok_or_else(HttpResponse::BadRequest)?;
    let variables = query.get("variables").ok_or_else(HttpResponse::BadRequest)?;

    let graphql_response = handler.execute(session, graphql_query, variables);
    Ok(HttpResponse::Ok().content_type("application/json").body(graphql_response))
}

pub fn app_configure(config: &mut ServiceConfig, server_data: Arc<ServerData>) {
    config.data(Arc::clone(&server_data)).service(
        web::resource("/{module_name}/graphql")
            .route(web::post().to(handle_request))
            .route(web::get().to(handle_request)),
    );
}

pub fn run_server(server_data: ServerData, addr: SocketAddr) -> Result<Server> {
    let server_data = Arc::new(server_data);
    let server = HttpServer::new(move || {
        App::new().configure(|config: &mut ServiceConfig| app_configure(config, Arc::clone(&server_data)))
    });
    Ok(server.bind(addr)?.run())
}
