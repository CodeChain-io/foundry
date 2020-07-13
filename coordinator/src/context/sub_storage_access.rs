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

use remote_trait_object::{service, Service};

// Interface between each module and the coordinator
#[service]
pub trait SubStorageAccess: Service {
    fn get(&self, key: &[u8]) -> Option<Vec<u8>>;
    fn set(&mut self, key: &[u8], value: Vec<u8>);
    fn has(&self, key: &[u8]) -> bool;
    fn remove(&mut self, key: &[u8]);

    /// Create a recoverable checkpoint of this state
    fn create_checkpoint(&mut self);
    /// Revert to the last checkpoint and discard it
    fn revert_to_the_checkpoint(&mut self);
    /// Merge last checkpoint with the previous
    fn discard_checkpoint(&mut self);
}
