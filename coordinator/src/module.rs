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

//! A set of service traits expected to be implemented and exported by Foundry modules.
//!
//! Most of methods take a `SessionID` to specify which session to use.
//! Session is an identifier which represents a state at a certain height of block.
//! All modules will be requested to perform given action based on speicific state
//! and passing that always explicitly makes the interface stateless.
//!
//! The consensus engine will request module to setup the session id with given `SubStorageAccess`
//! when it's needed (before/after tasks), using `Stateful`.
//! Managing mapping from session id to the actual proxy of `SubStorageAccess` is role of module.
//!
//! One of typical implementations of module would have a hashmap from `SessionId` to `dyn SubStorageAccess`s.
//! That can be accessed globally from other services.

use super::context::SubStorageAccess;
use crate::transaction::{Transaction, TransactionWithMetadata};
use crate::types::{CloseBlockError, ErrorCode, Event, HeaderError, TransactionOutcome, VerifiedCrime};
use crate::Header;
use ctypes::{ChainParams, CompactValidatorSet};
use remote_trait_object::{service, Service, ServiceRef};
use serde::{Deserialize, Serialize};

pub type SessionId = u32;

/// A service trait for the module which has its own state.
///
/// As explained in module-level documentaiton, it's role is managing the mapping from session id to `SubStorageAccess`.
///
/// Note that implementing `TxOwner` and implementing this are independent.
/// There could be a module which is statless but defines its own transaction.
/// Conversely, there could be also a module which is stateful but has no transaction.
#[service]
pub trait Stateful: Service {
    /// Sets storage for the given session id.
    fn new_session(&mut self, id: SessionId, storage: ServiceRef<dyn SubStorageAccess>);
    /// Removes the storage with the given id.
    fn end_session(&mut self, id: SessionId);
}

/// A service to initialize the genesis state.
///
/// Any module can implement and export this if it has something to store in the state in the genesis phase.
#[service]
pub trait InitGenesis: Service {
    /// Sets up the genesis state.
    fn init_genesis(&self, session_id: SessionId, config: &[u8]);
}

/// A service trait for transaction executions.
///
/// Every modules who define transactions must implement and export an instance of this service.
/// It is tentative and will be changed after
/// https://github.com/CodeChain-io/foundry/issues/555 and
/// https://github.com/CodeChain-io/foundry/issues/556.
#[service]
pub trait TxOwner: Service {
    /// Prepares a block to execute.
    ///
    /// You can use `header`, which is of the previous block.
    fn block_opened(&self, session_id: SessionId, header: &Header) -> Result<(), HeaderError>;

    /// Executes a transaction.
    ///
    /// - If the transaction is invalid, it should return `Err`. All changes made upon the state will be reverted.
    /// - If the transaction is valid, it should return `Ok`. You might emit some events as a result.
    fn execute_transaction(&self, session_id: SessionId, transaction: &Transaction) -> Result<TransactionOutcome, ()>;

    /// Performs a lightweight check for the given transaction.
    ///
    /// As you can notice, it doesn't take `session: SessionId` like other methods.
    /// It is only for the stateless check such as format and signature, and is ok not to be perfect.
    /// Even it can always return Ok(()). However, it must never return an error for (possibly) valid transaction.
    /// That will make the node Byzantine.
    ///
    /// This will be mainly for the mempool management.
    fn check_transaction(&self, transaction: &Transaction) -> Result<(), ErrorCode>;

    /// Closes the block opend by `block_opened()`.
    ///
    /// Like `execute_transaction()`, you can return events if there are some.
    fn block_closed(&self, session_id: SessionId) -> Result<Vec<Event>, CloseBlockError>;
}

/// A service to initiliaze the genesis consensus paramters
///
/// It decides both validator set and chain parameters.
/// They are considered as _consensus parameters_ which affect the behavior of the consens engine.
/// Especially choosing a validator set based on a sound staking rule is essential to keep the PoS chain safe.
///
/// Unlike `InitGenesis`, it must be exported from single module.
#[service]
pub trait InitConsensus: Service {
    /// Decides the initial consensus parmaters and returns them.
    fn init_consensus(&self, session_id: SessionId) -> (CompactValidatorSet, ChainParams);
}

/// A service to update the genesis consensus paramters
///
/// Like `InitChain`, it decides the consensus parameters
/// but is called every block right after calling `block_closed()` for every modules.
///
/// Similarily to `InitConsensus`, it must be exported from single module. (Mostly the same module with `InitConsensus`)
#[service]
pub trait UpdateConsensus: Service {
    /// Decides the consensus parmaters for this block and returns them.
    fn update_consensus(&self, session_id: SessionId) -> (Option<CompactValidatorSet>, Option<ChainParams>);
}

/// A service to sort transactions.
#[service]
pub trait TxSorter: Service {
    /// Sorts transactions.
    ///
    /// The implementor of this service will encounter all kinds of transactions in the application,
    /// and should sort them to maximize the benefit of node operator.
    ///
    /// A 'benefit' can be from various aspects, depending on the stragey of node operator's.
    /// We ENCOURAGE to implement your own module to tune the sorting behavior.
    ///
    /// Like `check_transaction()`, it doesn't have to be perfect on its invalidity checking but must never
    /// report a (possibly) valid transaction to be invalid.
    fn sort_txs(&self, session_id: SessionId, txs: &[TransactionWithMetadata]) -> SortedTxs;
}

/// The result from `sort_txs()`.
#[derive(Serialize, Deserialize, Default)]
pub struct SortedTxs {
    pub invalid: Vec<usize>,
    pub sorted: Vec<usize>,
}

/// A service to handle crimes reported from the consensus engine.
#[service]
pub trait HandleCrimes: Service {
    /// Handles the crimes.
    ///
    /// The crime is verified by the consensus engine, which is a misbehavior in Tendermint processs.
    /// It updates the state based on the policy of the application.
    fn handle_crimes(&self, session_id: SessionId, crimes: &[VerifiedCrime]);
}

/// A service to handle GraphQL requests.
#[service]
pub trait HandleGraphQlRequest: Service {
    fn execute(&self, session_id: SessionId, query: &str, variables: &str) -> String;
}
