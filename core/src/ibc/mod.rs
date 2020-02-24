// Copyright 2019-2020 Kodebox, Inc.
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

pub mod client_02;
#[allow(dead_code)]
#[allow(unused_variables)]
pub mod commitment_23;
#[allow(dead_code)]
#[allow(unused_variables)]
pub mod connection_03;
pub mod context;
mod kv_store;
pub mod querier;
mod transaction_handler;

pub use self::client_02 as client;
pub use self::context::Context;
pub use self::kv_store::KVStore;
pub use transaction_handler::execute as execute_transaction;

/// Widely used in IBC. In most case it will be part of a state DB path.
pub type Identifier = String;
pub type IdentifierSlice<'a> = &'a str;
