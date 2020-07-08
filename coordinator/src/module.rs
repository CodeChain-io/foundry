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

use super::context::SubStorageAccess;
use super::types::*;
use ctypes::{CompactValidatorSet, ConsensusParams};
use remote_trait_object::{Service, ServiceRef};

#[remote_trait_object_macro::service]
pub trait Stateful: Service {
    fn set_storage(&mut self, storage: ServiceRef<dyn SubStorageAccess>);
}

#[remote_trait_object_macro::service]
pub trait InitGenesis: Service {
    fn begin_genesis(&self);

    fn init_genesis(&mut self, config: &[u8]);

    fn end_genesis(&self);
}

#[remote_trait_object_macro::service]
pub trait TxOwner: Service {
    fn block_opened(&self) -> Result<(), HeaderError>;

    fn execute_transaction(&mut self, transaction: &Transaction) -> Result<TransactionExecutionOutcome, ()>;

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;

    fn block_closed(&self) -> Result<Vec<Event>, CloseBlockError>;
}

#[remote_trait_object_macro::service]
pub trait InitChain: Service {
    fn init_chain(&self) -> (CompactValidatorSet, ConsensusParams);
}

#[remote_trait_object_macro::service]
pub trait UpdateChain: Service {
    fn update_chain(&self) -> (CompactValidatorSet, ConsensusParams);
}
