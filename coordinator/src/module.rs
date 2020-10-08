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
use ctypes::{ChainParams, CompactValidatorSet};
use remote_trait_object::{service, Service, ServiceRef};
use serde::{Deserialize, Serialize};

pub type SessionId = u32;

#[service]
pub trait Stateful: Service {
    fn new_session(&mut self, id: SessionId, storage: ServiceRef<dyn SubStorageAccess>);

    fn end_session(&mut self, id: SessionId);
}

#[service]
pub trait InitGenesis: Service {
    fn init_genesis(&self, session_id: SessionId, config: &[u8]);
}

#[service]
pub trait TxOwner: Service {
    fn block_opened(&self, session_id: SessionId, header: &Header) -> Result<(), HeaderError>;

    fn execute_transaction(&self, session_id: SessionId, transaction: &Transaction) -> Result<TransactionOutcome, ()>;

    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;

    fn block_closed(&self, session_id: SessionId) -> Result<Vec<Event>, CloseBlockError>;
}

#[service]
pub trait InitChain: Service {
    fn init_chain(&self, session_id: SessionId) -> (CompactValidatorSet, ChainParams);
}

#[service]
pub trait UpdateChain: Service {
    fn update_chain(&self, session_id: SessionId) -> (Option<CompactValidatorSet>, Option<ChainParams>);
}

#[service]
pub trait TxSorter: Service {
    fn sort_txs(&self, session_id: SessionId, txs: &[TransactionWithMetadata]) -> SortedTxs;
}

#[derive(Serialize, Deserialize, Default)]
pub struct SortedTxs {
    pub invalid: Vec<usize>,
    pub sorted: Vec<usize>,
}

#[service]
pub trait HandleCrimes: Service {
    fn handle_crimes(&self, session_id: SessionId, crimes: &[VerifiedCrime]);
}

#[service]
pub trait HandleGraphQlRequest: Service {
    fn execute(&self, session_id: SessionId, query: &str, variables: &str) -> String;
}
