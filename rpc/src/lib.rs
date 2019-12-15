// Copyright 2018-2019 Kodebox, Inc.
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

extern crate codechain_core as ccore;
extern crate codechain_crypto as ccrypto;
extern crate codechain_json as cjson;
extern crate codechain_key as ckey;
extern crate codechain_keystore as ckeystore;
extern crate codechain_logger as clogger;
extern crate codechain_network as cnetwork;
extern crate codechain_state as cstate;
extern crate codechain_sync as csync;
extern crate codechain_types as ctypes;
#[macro_use]
extern crate lazy_static;
extern crate log;
#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate jsonrpc_derive;

use cidr;
pub use jsonrpc_core;
use jsonrpc_http_server;

pub mod rpc_server;
pub mod v1;

pub use rustc_serialize::hex;

pub use jsonrpc_core::{Compatibility, Error, MetaIoHandler, Middleware, Params, Value};

pub use jsonrpc_http_server::Server as HttpServer;
pub use rpc_server::start_http;

pub use jsonrpc_ipc_server::Server as IpcServer;
pub use rpc_server::start_ipc;

pub use jsonrpc_ws_server::{Error as WsError, Server as WsServer};
pub use rpc_server::start_ws;
