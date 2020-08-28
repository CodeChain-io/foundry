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
use parking_lot::RwLock;
use std::sync::Arc;

pub trait StateAccess {
    type Outcome;
    fn execute(self, state: &dyn SubStorageAccess) -> Self::Outcome;
}

pub trait StateTransition {
    type Outcome;
    fn execute(self, state: &mut dyn SubStorageAccess) -> Self::Outcome;
}

/// A struct that defines the way of accessing the state and the way of making state transition.
pub struct StateMachine {
    /// The state that this machine will act upon.
    state: Arc<RwLock<dyn SubStorageAccess>>,
}

impl StateMachine {
    pub fn new(state: Arc<RwLock<dyn SubStorageAccess>>) -> Self {
        Self {
            state,
        }
    }

    pub fn execute_access<S: StateAccess>(&self, x: S) -> S::Outcome {
        x.execute(&*(self.state.read()))
    }

    pub fn execute_transition<S: StateTransition>(&self, x: S) -> S::Outcome {
        x.execute(&mut *(self.state.write()))
    }
}
