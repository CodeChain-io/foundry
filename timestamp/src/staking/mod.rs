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
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

mod module;
pub mod services;

use crate::common::*;
use crate::token::services::TokenManager;
pub use ckey::Ed25519Public as Public;
use coordinator::Transaction;
pub use module::Module;
use parking_lot::RwLock;
use primitives::H256;
use remote_trait_object::raw_exchange::import_null_proxy;
use remote_trait_object::{service, Service};

struct Config {
    pub validator_token_issuer: H256,
}

#[service]
pub trait GetAccountAndSeq: Service {
    fn get_account_and_seq(&self, tx: &Transaction) -> Result<(Public, TxSeq), ()>;
}

struct ServiceHandler {
    token_manager: RwLock<Box<dyn TokenManager>>,
    config: Config,
}

impl ServiceHandler {
    fn new(config: Config) -> Self {
        Self {
            token_manager: RwLock::new(import_null_proxy()),
            config,
        }
    }
}

impl remote_trait_object::Service for ServiceHandler {}
