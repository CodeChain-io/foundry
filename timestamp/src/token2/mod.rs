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

mod graphql;
mod module;
pub mod services;
mod state_machine;
mod types;

use super::common::state_machine::StateMachine;
use super::common::StateManager;
use coordinator::module2::{SessionKey, Stateful};
use parking_lot::RwLock;
use std::sync::Arc;

struct ServiceHandler {
    state_manager: Arc<RwLock<StateManager>>,

    account_manager: RwLock<Box<dyn crate::account2::services::AccountManager>>,
}

impl ServiceHandler {
    fn new() -> Self {
        Self {
            state_manager: Arc::new(RwLock::new(StateManager::default())),
            account_manager: RwLock::new(remote_trait_object::raw_exchange::import_null_proxy()),
        }
    }

    fn account_manager(&self) -> &RwLock<Box<dyn crate::account2::services::AccountManager>> {
        &self.account_manager
    }

    fn create_state_machine(&self, session: SessionKey) -> StateMachine {
        StateMachine::new(self.state_manager.read().get(session))
    }

    fn get_stateful(&self) -> Arc<RwLock<dyn Stateful>> {
        Arc::clone(&self.state_manager) as Arc<RwLock<dyn Stateful>>
    }
}

impl remote_trait_object::Service for ServiceHandler {}

pub use types::Error;
