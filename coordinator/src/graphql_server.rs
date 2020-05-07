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

use actix_cors::Cors;
use actix_web::{
    dev::Server,
    error::{ErrorMethodNotAllowed, ErrorUnauthorized, ErrorUnsupportedMediaType},
    http::{header::CONTENT_TYPE, Method},
    web,
    web::Payload,
    App, FromRequest, HttpRequest, HttpResponse, HttpServer, Result,
};
use juniper::http::graphiql::graphiql_source;
use queryst::parse;
use std::collections::HashMap;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
pub trait GraphQLHandler: Send + Sync {
    fn execute(&self, request: &str) -> String;
}

pub type HandlerMap = HashMap<String, Box<dyn GraphQLHandler>>;

async fn handle_request(
    request: HttpRequest,
    payload: Payload,
    path: web::Path<String>,
    handlers: web::Data<Arc<HandlerMap>>,
) -> Result<HttpResponse> {
    let module_name = path.into_inner();
    if let Some(handler) = handlers.get(&module_name) {
        match *request.method() {
            Method::POST => handle_post(request, payload, handler.deref()).await,
            Method::GET => handle_get(request, payload, handler.deref()).await,
            _ => Err(ErrorMethodNotAllowed("GraphQL requests can only be sent with GET or POST")),
        }
    } else {
        Err(ErrorUnauthorized(format!("Cannot access /{}/graphql", module_name)))
    }
}

async fn handle_graphiql(path: web::Path<String>) -> Result<HttpResponse> {
    let module_name = path.into_inner();
    let graphql_endpoint_url = format! {"/{}/graphql", module_name};
    let html = graphiql_source(&graphql_endpoint_url);
    Ok(HttpResponse::Ok().content_type("text/html; charset=utf-8").body(html))
}

async fn handle_post(request: HttpRequest, payload: Payload, handler: &dyn GraphQLHandler) -> Result<HttpResponse> {
    let content_type_header = request.headers().get(CONTENT_TYPE).and_then(|hv| hv.to_str().ok());
    let gql_request = match content_type_header {
        Some("application/json") | Some("application/graphql") => {
            String::from_request(&request, &mut payload.into_inner()).await
        }
        _ => Err(ErrorUnsupportedMediaType(
            "GraphQL requests should have content type `application/json` or `application/graphql`",
        )),
    }?;

    let gql_response = handler.execute(&gql_request);
    // Q. Do we need to return HttpResponse::BadRequest()?
    // Currently, the coordinator cannot determine that
    Ok(HttpResponse::Ok().content_type("application/json").body(gql_response))
}

async fn handle_get(request: HttpRequest, payload: Payload, handler: &dyn GraphQLHandler) -> Result<HttpResponse> {
    let graphql_request = request.query_string();
    let value = parse(graphql_request).map_err(|_| HttpResponse::BadRequest())?;
    let graphql_response = handler.execute(&value.to_string());
    Ok(HttpResponse::Ok().content_type("application/json").body(graphql_response))
}

pub async fn run_graphql(graphql_handlers: Arc<HandlerMap>, addr: SocketAddr) -> Result<Server> {
    let server = HttpServer::new(move || {
        App::new()
            .data(graphql_handlers.clone())
            .wrap(Cors::new().allowed_methods(vec!["POST", "GET"]).finish())
            .service(
                web::resource("/{module_name}/graphql")
                    .route(web::post().to(handle_request))
                    .route(web::get().to(handle_request)),
            )
            .service(web::resource("/{module_name}/__graphql").route(web::get().to(handle_graphiql)))
    });
    Ok(server.bind(addr)?.run())
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_rt;
    use actix_web::{http, test, web, App};
    use std::str;

    mod juniper_test {
        use super::*;
        use juniper::{
            http::GraphQLRequest,
            tests::{model::Database, schema::Query},
            EmptyMutation, RootNode,
        };

        #[actix_rt::test]
        async fn test() {
            use std::net::{IpAddr, Ipv4Addr, SocketAddr};

            let socket = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8080);
            let server = run_graphql(graphql_handlers(), socket).await;
        }

        #[actix_rt::test]
        async fn test_get() {
            let mut app = test::init_service(
                App::new().data(graphql_handlers()).service(
                    web::resource("/{module_name}/graphql")
                        .route(web::post().to(handle_request))
                        .route(web::get().to(handle_request)),
                ),
            )
            .await;
            let req = test::TestRequest::get().uri(r#"/sample_module/graphql?query={hero{name}}"#).to_request();
            let resp_bytes = test::read_response(&mut app, req).await;
            let resp = str::from_utf8(&resp_bytes).expect("GraphQL server must return utf8-encoded string");
            assert_eq!(resp, "{\"data\":{\"hero\":{\"name\":\"R2-D2\"}}}");
        }

        #[actix_rt::test]
        async fn test_post() {
            let mut app = test::init_service(
                App::new().data(graphql_handlers()).service(
                    web::resource("/{module_name}/graphql")
                        .route(web::post().to(handle_request))
                        .route(web::get().to(handle_request)),
                ),
            )
            .await;
            let req = test::TestRequest::post()
                .header("content-type", "application/json")
                .uri("/sample_module/graphql")
                .set_payload(r#"{"query": "{hero{name}}"}"#)
                .to_request();
            let resp_bytes = test::read_response(&mut app, req).await;
            let resp = str::from_utf8(&resp_bytes).expect("GraphQL server must return utf8-encoded string");
            assert_eq!(resp, "{\"data\":{\"hero\":{\"name\":\"R2-D2\"}}}");
        }

        #[actix_rt::test]
        async fn test_reject_unregistered_path() {
            let mut app = test::init_service(
                App::new().data(graphql_handlers()).service(
                    web::resource("/{module_name}/graphql")
                        .route(web::post().to(handle_request))
                        .route(web::get().to(handle_request)),
                ),
            )
            .await;
            let req = test::TestRequest::default().uri("/unregistered_module/graphql").to_request();
            let resp = test::call_service(&mut app, req).await;
            assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
        }

        type Schema = juniper::RootNode<'static, Query, EmptyMutation<Database>>;

        struct Module {
            schema: Schema,
            database: Database,
        }

        impl GraphQLHandler for Module {
            fn execute(&self, request: &str) -> String {
                let graphql_request = match serde_json::from_str::<GraphQLRequest>(request) {
                    Ok(request) => request,
                    // Should return `HTTPError::BadRequest`
                    // Should we change the return type of this method into `Result<String, Error>`?
                    Err(e) => return e.to_string(),
                };
                let graphql_response = graphql_request.execute(&self.schema, &self.database);
                serde_json::to_string(&graphql_response).unwrap()
            }
        }

        fn create_graphql_handler() -> Box<dyn GraphQLHandler> {
            let schema: Schema = RootNode::new(Query, EmptyMutation::<Database>::new());
            Box::new(Module {
                schema,
                database: Database::new(),
            })
        }

        fn graphql_handlers() -> Arc<HandlerMap> {
            Arc::new(
                (vec![("sample_module".to_string(), create_graphql_handler())]).into_iter().collect::<HashMap<_, _>>(),
            )
        }
    }

    mod async_graphql_test {
        use super::*;

        use async_graphql::http::{GQLRequest, GQLResponse};
        use async_graphql::{EmptyMutation, EmptySubscription, IntoQueryBuilder, Schema};
        use futures::executor::block_on;
        use starwars::{QueryRoot, StarWars, StarWarsSchema};

        #[actix_rt::test]
        async fn test_get() {
            let mut app = test::init_service(
                App::new().data(graphql_handlers()).service(
                    web::resource("/{module_name}/graphql")
                        .route(web::post().to(handle_request))
                        .route(web::get().to(handle_request)),
                ),
            )
            .await;
            let req = test::TestRequest::get()
                .uri("/sample_module/graphql?query=query%20%7B%20human(id%3A%20%221000%22)%20%7B%20id%2C%20name%2C%20appearsIn%2C%20homePlanet%20%7D%20%7D")
                .to_request();
            let resp_bytes = test::read_response(&mut app, req).await;
            let resp = str::from_utf8(&resp_bytes).expect("GraphQL server must return utf8-encoded string");
            assert_eq!(resp, "{\"data\":{\"human\":{\"id\":\"1000\",\"name\":\"Luke Skywalker\",\"appearsIn\":[],\"homePlanet\":\"Tatooine\"}}}");
        }

        #[actix_rt::test]
        async fn test_post() {
            let mut app = test::init_service(
                App::new().data(graphql_handlers()).service(
                    web::resource("/{module_name}/graphql")
                        .route(web::post().to(handle_request))
                        .route(web::get().to(handle_request)),
                ),
            )
            .await;
            let req = test::TestRequest::post()
                .header("content-type", "application/json")
                .uri("/sample_module/graphql")
                .set_payload(r#"{"query": "{human(id:\"1000\"){name}}"}"#)
                .to_request();
            let resp_bytes = test::read_response(&mut app, req).await;
            let resp = str::from_utf8(&resp_bytes).expect("GraphQL server must return utf8-encoded string");
            assert_eq!(resp, "{\"data\":{\"human\":{\"name\":\"Luke Skywalker\"}}}");
        }

        struct Module {
            schema: StarWarsSchema,
        }

        impl GraphQLHandler for Module {
            fn execute(&self, request: &str) -> String {
                let gql_request = match serde_json::from_str::<GQLRequest>(request) {
                    Ok(gql_request) => gql_request,
                    Err(e) => return format!("{}", e),
                };
                let query_builder = match block_on(gql_request.into_query_builder()) {
                    Ok(query_builder) => query_builder,
                    Err(e) => return format!("{}", e),
                };
                let response = GQLResponse(block_on(query_builder.execute(&self.schema)));
                serde_json::to_string(&response).unwrap()
            }
        }

        fn create_graphql_handler() -> Box<dyn GraphQLHandler> {
            let schema = Schema::build(QueryRoot, EmptyMutation, EmptySubscription).data(StarWars::new()).finish();
            Box::new(Module {
                schema,
            })
        }

        fn graphql_handlers() -> Arc<HandlerMap> {
            Arc::new(
                (vec![("sample_module".to_string(), create_graphql_handler())]).into_iter().collect::<HashMap<_, _>>(),
            )
        }
    }
}
