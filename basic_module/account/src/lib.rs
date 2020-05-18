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

extern crate codechain_crypto as ccrypto;
extern crate codechain_key as ckey;

mod core;
mod error;
mod impls;
mod import;
mod internal;
mod types;

use ckey::verify;
use ckey::NetworkId;
use coordinator::context::Context;
use parking_lot::Mutex;
use std::unimplemented;
use types::SignedTransaction;

lazy_static! {
    static ref NETWORK_ID: Mutex<Option<NetworkId>> = Mutex::new(None);
}

pub fn get_context() -> &'static mut dyn Context {
    // This function should be implemented after the context has been formatted.
    unimplemented!();
}

pub fn check(signed_tx: &SignedTransaction) -> bool {
    let signature = signed_tx.signature;
    let network_id = signed_tx.tx.network_id;

    check_network_id(network_id) && verify(&signature, &signed_tx.tx.hash(), &signed_tx.signer_public)
}

pub fn check_network_id(network_id: NetworkId) -> bool {
    let mut saved_network_id = NETWORK_ID.lock();
    if saved_network_id.is_none() {
        *saved_network_id = Some(network_id);
    }
    *saved_network_id == Some(network_id)
}
