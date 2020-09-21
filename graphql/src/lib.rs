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
    error::{ErrorBadRequest, ErrorNotFound},
    web,
    web::ServiceConfig,
    App, Error, FromRequest, HttpRequest, HttpResponse, HttpServer, Result,
};
use coordinator::module::{HandleGraphQlRequest, SessionId};
use futures::Future;
pub use handler::handle_gql_query;
use serde::Deserialize;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::{pin::Pin, sync::Arc};

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

#[derive(Deserialize)]
struct GraphQlArgs {
    query: String,
    variables: Option<String>,
}

async fn handle_post(session_and_handler: SessionAndHandler, args: web::Json<GraphQlArgs>) -> Result<HttpResponse> {
    let SessionAndHandler {
        session,
        handler,
    } = session_and_handler;
    let query = &args.query;
    let variables = args.variables.as_deref().unwrap_or("{}");

    let graphql_response = handler.execute(session, query, variables);
    Ok(HttpResponse::Ok().content_type("application/json").body(graphql_response))
}

async fn handle_get(session_and_handler: SessionAndHandler, args: web::Query<GraphQlArgs>) -> Result<HttpResponse> {
    let SessionAndHandler {
        session,
        handler,
    } = session_and_handler;
    let query = &args.query;
    let variables = args.variables.as_deref().unwrap_or("{}");

    let graphql_response = handler.execute(session, query, variables);
    Ok(HttpResponse::Ok().content_type("application/json").body(graphql_response))
}

pub fn app_configure(config: &mut ServiceConfig, server_data: Arc<ServerData>) {
    config.data(Arc::clone(&server_data)).service(
        web::resource("/{module_name}/graphql").route(web::post().to(handle_post)).route(web::get().to(handle_get)),
    );
}

pub fn run_server(server_data: ServerData, addr: SocketAddr) -> Result<Server> {
    let server_data = Arc::new(server_data);
    let server = HttpServer::new(move || {
        App::new().configure(|config: &mut ServiceConfig| app_configure(config, Arc::clone(&server_data)))
    });
    Ok(server.bind(addr)?.run())
}

struct SessionAndHandler {
    pub session: SessionId,
    pub handler: Arc<dyn HandleGraphQlRequest>,
}

impl FromRequest for SessionAndHandler {
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self, Self::Error>>>>;
    type Config = ();

    fn from_request(req: &HttpRequest, _payload: &mut actix_http::Payload) -> Self::Future {
        let module_name = req.match_info().get("module_name").map(|string| string.to_owned());
        let server_data = req.app_data::<web::Data<Arc<ServerData>>>().unwrap().clone();
        Box::pin(async move {
            let module_name = module_name.ok_or_else(|| ErrorBadRequest("module_name not found"))?;

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

                Ok(SessionAndHandler {
                    session,
                    handler: handler.clone(),
                })
            } else {
                Err(ErrorNotFound(format!("Module not found: {}", module_name)))
            }
        })
    }
}
