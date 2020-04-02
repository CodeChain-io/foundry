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
#[macro_use]
extern crate codechain_logger as clogger;
#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

extern crate rand;

pub mod handler;
mod informer_service;
pub mod rpc_server;

pub use cinfo_courier::{informer_notify, EventTags, Events, InformerEventSender};
pub use handler::{Connection, InformerConfig};
pub use informer_service::{ColdEvents, InformerService};
pub use jsonrpc_core;
pub use jsonrpc_core::{Compatibility, Error, ErrorCode, MetaIoHandler, Metadata, Middleware, Params, Value};
pub use jsonrpc_pubsub::{PubSubHandler, PubSubMetadata, Session, Sink, Subscriber, SubscriptionId};
pub use jsonrpc_ws_server::{Error as WsError, RequestContext, Server as WsServer, ServerBuilder as WsServerBuilder};
pub use rand::Rng;
pub use rpc_server::start_ws;
