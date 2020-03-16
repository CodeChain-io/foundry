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

use crate::{
    start_ws, Connection, Error, ErrorCode, Params, PubSubHandler, Rng, Session, Subscriber, SubscriptionId, Value,
    WsError, WsServer,
};
use crossbeam::Sender;
use crossbeam_channel as crossbeam;
use jsonrpc_core::{futures, BoxFuture};
use std::io;
use std::sync::Arc;

pub struct InformerConfig {
    pub interface: String,
    pub port: u16,
    pub max_connections: usize,
}

pub struct Handler {
    pub handler: PubSubHandler<Arc<Session>>,
}

impl Handler {
    pub fn new(handler: PubSubHandler<Arc<Session>>) -> Self {
        Self {
            handler,
        }
    }
    pub fn start_ws(self, config: InformerConfig) -> Result<WsServer, String> {
        let url = format!("{}:{}", config.interface, config.port);
        let addr = url.parse().map_err(|_| format!("Invalid WebSockets listen host/port given: {}", url))?;
        let server = self.handler;
        let start_result = start_ws(&addr, server, config.max_connections);
        match start_result {
            Err(WsError::Io(ref err)) if err.kind() == io::ErrorKind::AddrInUse => {
                Err(format!("WebSockets address {} is already in use, make sure that another instance of a Codechain node is not using this", addr))
            },
            Err(e) => {
                Err(format!("WebSockets error: {:?}", e))
            },
            Ok(server) => {
                cinfo!(INFORMER, "WebSockets Listening on {}", addr);
                Ok(server)
            },
        }
    }
    pub fn event_subscription(&mut self, sender: Sender<Connection>) {
        self.handler.add_subscription(
            "register",
            ("register", move |params: Params, _, subscriber: Subscriber| {
                if params == Params::None {
                    subscriber
                        .reject(Error {
                            code: ErrorCode::ParseError,
                            message: "Invalid parameters. Subscription rejected.".into(),
                            data: None,
                        })
                        .expect("Connection is alive");
                    return
                }
                let parsed_params: Result<Vec<String>, Error> = params.parse();
                let all_params = match parsed_params {
                    Ok(params) => params,
                    Err(_err) => {
                        subscriber
                            .reject(Error {
                                code: ErrorCode::ParseError,
                                message: "Invalid parameters. Subscription rejected.".into(),
                                data: None,
                            })
                            .expect("Connection is alive");
                        return
                    }
                };
                let mut rng = rand::thread_rng();
                let sub_id = rng.gen();
                let sink = subscriber.assign_id(SubscriptionId::Number(sub_id)).expect("Connection is alive");
                let mut connection = Connection::new(sink, sub_id);
                connection.add_events(all_params);
                sender.send(connection).unwrap();
            }),
            // FIXME: We need another channel to remove connections form informer Service after Deregister
            ("deregister", |_id: SubscriptionId, _meta| -> BoxFuture<Value> {
                cinfo!(INFORMER, "Closing subscription");
                Box::new(futures::future::ok(Value::Bool(true)))
            }),
        );
    }
}
