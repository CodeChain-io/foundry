// Copyright 2018-2020 Kodebox, Inc.
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

#![cfg_attr(feature = "nightly", feature(test))]

#[macro_use]
extern crate codechain_logger as clogger;
#[macro_use]
extern crate rlp_derive;
#[macro_use]
extern crate log;

mod account_provider;
pub mod block;
mod blockchain;
mod blockchain_info;
mod client;
mod consensus;
mod db;
mod db_version;
pub mod encoded;
mod error;
mod event;
pub mod genesis;
mod miner;
mod peer_db;
mod service;
mod transaction;
mod types;
mod verification;
mod views;

#[cfg(test)]
mod tests;

pub use crate::account_provider::{AccountProvider, Error as AccountProviderError};
pub use crate::block::Block;
pub use crate::client::snapshot_notify;
pub use crate::client::ConsensusClient;
pub use crate::client::{
    BlockChainClient, BlockChainTrait, ChainNotify, Client, ClientConfig, DatabaseClient, EngineClient, EngineInfo,
    ImportBlock, MiningBlockChainClient, SnapshotClient, StateInfo, TestBlockChainClient,
};
pub use crate::consensus::signer::EngineSigner;
pub use crate::consensus::tendermint::Evidence;
pub use crate::consensus::{ConsensusEngine, EngineType, NullEngine, Solo, Tendermint, TimeGapParams};
pub use crate::db::{COL_STATE, NUM_COLUMNS};
pub use crate::error::{BlockImportError, Error, ImportError};
pub use crate::miner::{Miner, MinerOptions, MinerService};
pub use crate::peer_db::PeerDb;
pub use crate::service::ClientService;
pub use crate::transaction::{LocalizedTransaction, PendingTransactions};
pub use crate::types::{BlockStatus, TransactionId};
pub use rlp::Encodable;
pub use views::{BlockView, BodyView, HeaderView};
