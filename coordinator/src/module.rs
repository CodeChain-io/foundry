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
use crate::transaction::{Transaction, TransactionWithMetadata};
use crate::types::{CloseBlockError, ErrorCode, Event, HeaderError, TransactionOutcome, VerifiedCrime};
use crate::Header;
use ctypes::{CompactValidatorSet, ConsensusParams};
use remote_trait_object::{Service, ServiceRef};
use serde::{Deserialize, Serialize};

#[remote_trait_object_macro::service]
pub trait Stateful: Service {
    fn set_storage(&mut self, storage: ServiceRef<dyn SubStorageAccess>);
}

#[remote_trait_object_macro::service]
pub trait InitGenesis: Service {
    fn begin_genesis(&mut self);

    fn init_genesis(&mut self, config: &[u8]);

    fn end_genesis(&mut self);
}

#[remote_trait_object_macro::service]
pub trait TxOwner: Service {
    fn block_opened(&mut self, header: &Header) -> Result<(), HeaderError>;

    fn execute_transaction(&mut self, transaction: &Transaction) -> Result<TransactionOutcome, ()>;

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;

    fn block_closed(&mut self) -> Result<Vec<Event>, CloseBlockError>;
}

#[remote_trait_object_macro::service]
pub trait InitChain: Service {
    fn init_chain(&mut self) -> (CompactValidatorSet, ConsensusParams);
}

#[remote_trait_object_macro::service]
pub trait UpdateChain: Service {
    fn update_chain(&mut self) -> (Option<CompactValidatorSet>, Option<ConsensusParams>);
}

#[remote_trait_object_macro::service]
pub trait TxSorter: Service {
    fn sort_txs(&self, txs: &[TransactionWithMetadata]) -> SortedTxs;
}

#[derive(Serialize, Deserialize, Default)]
pub struct SortedTxs {
    pub invalid: Vec<usize>,
    pub sorted: Vec<usize>,
}

#[remote_trait_object_macro::service]
pub trait HandleCrimes: Service {
    fn handle_crimes(&mut self, crimes: &[VerifiedCrime]);
}
