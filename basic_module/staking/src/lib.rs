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
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use coordinator::context::{ChainHistoryAccess, SubStateHistoryAccess, SubStorageAccess};
use imported::{AccountManager, AccountView, FeeManager};

mod check;
mod core;
mod error;
mod execute;
mod impls;
mod imported;
mod runtime_error;
mod state;
mod syntax_error;
mod transactions;
mod types;

pub fn substorage() -> Box<dyn SubStorageAccess> {
    unimplemented!()
}

pub fn deserialize<T: serde::de::DeserializeOwned>(buffer: Vec<u8>) -> T {
    serde_cbor::from_slice(&buffer).unwrap()
}

pub fn serialize<T: serde::ser::Serialize>(data: T) -> Vec<u8> {
    serde_cbor::to_vec(&data).unwrap()
}

// FIXME: network_id should be mutable
lazy_static! {
    static ref NETWORK_ID: types::NetworkId = Default::default();
}

fn check_network_id(network_id: types::NetworkId) -> bool {
    *NETWORK_ID == network_id
}

pub fn account_manager() -> Box<dyn AccountManager> {
    unimplemented!()
}

pub fn account_viewer() -> Box<dyn AccountView> {
    unimplemented!()
}

pub fn fee_manager() -> Box<dyn FeeManager> {
    unimplemented!()
}

pub fn chain_history_manager() -> Box<dyn ChainHistoryAccess> {
    unimplemented!()
}

pub fn state_history_manager() -> Box<dyn SubStateHistoryAccess> {
    unimplemented!()
}
