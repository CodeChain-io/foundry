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

mod module;
pub mod services;
mod types;

use parking_lot::RwLock;
use primitives::H256;
use remote_trait_object::raw_exchange::import_null_proxy;

struct Config {
    token_issuer: H256,
}

struct ServiceHandler {
    config: Config,
    account_manager: RwLock<Box<dyn crate::account2::services::AccountManager>>,
    token_manager: RwLock<Box<dyn crate::token2::services::TokenManager>>,
}

impl ServiceHandler {
    fn new(config: Config) -> Self {
        Self {
            config,
            account_manager: RwLock::new(import_null_proxy()),
            token_manager: RwLock::new(import_null_proxy()),
        }
    }

    fn account_manager(&self) -> &RwLock<Box<dyn crate::account2::services::AccountManager>> {
        &self.account_manager
    }

    fn token_manager(&self) -> &RwLock<Box<dyn crate::token2::services::TokenManager>> {
        &self.token_manager
    }
}

impl remote_trait_object::Service for ServiceHandler {}
