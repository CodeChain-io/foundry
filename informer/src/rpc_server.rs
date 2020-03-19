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

use crate::{PubSubHandler, RequestContext, Session, WsError, WsServer, WsServerBuilder};
use std::net::SocketAddr;
use std::sync::Arc;

pub fn start_ws(
    addr: &SocketAddr,
    handler: PubSubHandler<Arc<Session>>,
    max_connections: usize,
) -> Result<WsServer, WsError> {
    WsServerBuilder::with_meta_extractor(handler, |context: &RequestContext| Arc::new(Session::new(context.sender())))
        .max_connections(max_connections)
        .start(addr)
}
