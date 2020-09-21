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

#[macro_use]
extern crate serde_derive;

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

use crate::types::NetworkId;
use coordinator::context::{ChainHistoryAccess, SubStorageAccess};
use imported::{AccountManager, AccountView};
use lazy_static::lazy_static;
use parking_lot::Mutex;

fn substorage() -> Box<dyn SubStorageAccess> {
    unimplemented!()
}

fn deserialize<T: serde::de::DeserializeOwned>(buffer: Vec<u8>) -> T {
    serde_cbor::from_slice(&buffer).unwrap()
}

fn serialize<T: serde::ser::Serialize>(data: T) -> Vec<u8> {
    serde_cbor::to_vec(&data).unwrap()
}

lazy_static! {
    static ref NETWORK_ID: Mutex<Option<NetworkId>> = Default::default();
}

fn check_network_id(network_id: NetworkId) -> bool {
    let mut saved_network_id = NETWORK_ID.lock();
    if let Some(saved_network_id) = *saved_network_id {
        return saved_network_id == network_id
    }
    *saved_network_id = Some(network_id);
    true
}

fn account_manager() -> Box<dyn AccountManager> {
    unimplemented!()
}

fn account_viewer() -> Box<dyn AccountView> {
    unimplemented!()
}

fn chain_history_manager() -> Box<dyn ChainHistoryAccess> {
    unimplemented!()
}
