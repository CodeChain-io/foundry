// Copyright 2018-2020 Kodebox, Inc.
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

#![cfg_attr(feature = "nightly", feature(test))]

extern crate codechain_crypto as ccrypto;
extern crate codechain_db as cdb;
extern crate codechain_io as cio;
extern crate codechain_json as cjson;
extern crate codechain_key as ckey;
extern crate codechain_keystore as ckeystore;
#[macro_use]
extern crate codechain_logger as clogger;
extern crate codechain_network as cnetwork;
extern crate codechain_state as cstate;
extern crate codechain_timer as ctimer;
extern crate codechain_types as ctypes;
extern crate codechain_vm as cvm;
#[cfg(test)]
extern crate rand_xorshift;
extern crate rlp;
#[macro_use]
extern crate rlp_derive;
#[macro_use]
extern crate log;

mod account_provider;
pub mod block;
mod blockchain;
mod blockchain_info;
mod client;
mod codechain_machine;
mod consensus;
mod db;
mod db_version;
pub mod encoded;
mod error;
mod invoice;
mod miner;
mod peer_db;
mod scheme;
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
    AccountData, BlockChainClient, BlockChainTrait, ChainNotify, Client, ClientConfig, DatabaseClient, EngineClient,
    EngineInfo, ExecuteClient, ImportBlock, MiningBlockChainClient, Shard, SnapshotClient, StateInfo, TermInfo,
    TestBlockChainClient,
};
pub use crate::consensus::signer::EngineSigner;
pub use crate::consensus::stake;
pub use crate::consensus::{EngineType, TimeGapParams};
pub use crate::db::{COL_PEER, COL_STATE, NUM_COLUMNS};
pub use crate::error::{BlockImportError, Error, ImportError};
pub use crate::miner::{MemPoolFees, Miner, MinerOptions, MinerService};
pub use crate::peer_db::PeerDb;
pub use crate::rlp::Encodable;
pub use crate::scheme::Scheme;
pub use crate::service::ClientService;
pub use crate::transaction::{
    LocalizedTransaction, PendingSignedTransactions, SignedTransaction, UnverifiedTransaction,
};
pub use crate::types::{BlockId, BlockStatus, TransactionId};
