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

use crate::{
    start_ws, Error, ErrorCode, Params, PubSubHandler, Rng, Session, Subscriber, Subscription, SubscriptionId, Value,
    WsError, WsServer,
};
use crossbeam::Sender;
use crossbeam_channel as crossbeam;
use jsonrpc_core::{futures, BoxFuture};
use std::io;
use std::net::Ipv4Addr;
use std::sync::Arc;

#[derive(Clone)]
pub enum Registration {
    Register(Subscription),
    Deregister(SubscriptionId),
}

pub struct InformerConfig {
    pub interface: Ipv4Addr,
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
    pub fn event_subscription(&mut self, sender: Sender<Registration>) {
        let register_sender = sender;
        let deregister_sender = register_sender.clone();
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
                let sub_id = Handler::next_id();
                let sink = subscriber.assign_id(SubscriptionId::Number(sub_id)).expect("Connection is alive");
                let mut subscription = Subscription::new(sink, SubscriptionId::Number(sub_id));
                subscription.add_events(all_params);
                let register = Registration::Register(subscription);
                register_sender.send(register).expect("The subscription channel is not full and also it is connected");
            }),
            ("deregister", move |id: SubscriptionId, _meta| -> BoxFuture<Value> {
                cinfo!(INFORMER, "Closing subscription");
                let deregister = Registration::Deregister(id);
                deregister_sender
                    .send(deregister)
                    .expect("The subscription cancellation channel is not full and also it is connected");
                Box::new(futures::future::ok(Value::Bool(true)))
            }),
        );
    }
    fn next_id() -> u64 {
        let mut rng = rand::thread_rng();
        rng.gen()
    }
}
