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

use coordinator::context::SubStorageAccess;
use coordinator::module2::{SessionKey, Stateful};
use parking_lot::RwLock;
use remote_trait_object::{Service, ServiceRef};
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default)]
pub struct StateManager {
    states: HashMap<SessionKey, Arc<RwLock<dyn SubStorageAccess>>>,
}

impl Service for StateManager {}

impl Stateful for StateManager {
    fn set_storage(&mut self, session: SessionKey, storage: ServiceRef<dyn SubStorageAccess>) {
        assert!(
            self.states.insert(session, storage.unwrap_import().into_proxy()).is_none(),
            "invalid set_storage() requested from coordinator. This is a bug"
        )
    }

    fn clear_storage(&mut self, session: SessionKey) {
        self.states.remove(&session).expect("invalid clear_storage() requested from coordinator. This is a bug");
    }
}

impl StateManager {
    pub fn get(&self, session: SessionKey) -> Arc<RwLock<dyn SubStorageAccess>> {
        Arc::clone(&self.states.get(&session).unwrap())
    }
}
