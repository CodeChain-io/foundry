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

use super::super::types::{Block, BlockNumberAndHash, Transaction, ValidatorSet};
use cjson::scheme::Params;
use ckey::{NetworkId, PlatformAddress};
use ctypes::{BlockHash, BlockNumber, TxHash};
use jsonrpc_core::Result;

#[rpc(server)]
pub trait Chain {
    /// Gets transaction with given hash.
    #[rpc(name = "chain_getTransaction")]
    fn get_transaction(&self, transaction_hash: TxHash) -> Result<Option<Transaction>>;

    /// Query whether the chain has the transaction with given transaction hash.
    #[rpc(name = "chain_containsTransaction")]
    fn contains_transaction(&self, transaction_hash: TxHash) -> Result<bool>;

    /// Gets number of best block.
    #[rpc(name = "chain_getBestBlockNumber")]
    fn get_best_block_number(&self) -> Result<BlockNumber>;

    /// Gets the number and the hash of the best block.
    #[rpc(name = "chain_getBestBlockId")]
    fn get_best_block_id(&self) -> Result<BlockNumberAndHash>;

    /// Gets the hash of the block with given number.
    #[rpc(name = "chain_getBlockHash")]
    fn get_block_hash(&self, block_number: u64) -> Result<Option<BlockHash>>;

    /// Gets block with given number.
    #[rpc(name = "chain_getBlockByNumber")]
    fn get_block_by_number(&self, block_number: u64) -> Result<Option<Block>>;

    /// Gets block with given hash.
    #[rpc(name = "chain_getBlockByHash")]
    fn get_block_by_hash(&self, block_hash: BlockHash) -> Result<Option<Block>>;

    ///Gets the count of transactions in a block with given hash.
    #[rpc(name = "chain_getBlockTransactionCountByHash")]
    fn get_block_transaction_count_by_hash(&self, block_hash: BlockHash) -> Result<Option<usize>>;

    /// Return the network id that is used in this chain.
    #[rpc(name = "chain_getNetworkId")]
    fn get_network_id(&self) -> Result<NetworkId>;

    /// Return common params at given block number
    #[rpc(name = "chain_getCommonParams")]
    fn get_common_params(&self, block_number: Option<u64>) -> Result<Option<Params>>;

    /// Return the current term id at given block number
    #[rpc(name = "chain_getTermMetadata")]
    fn get_term_metadata(&self, block_number: Option<u64>) -> Result<Option<(u64, u64)>>;

    /// Return the current metadata seq at given block number
    #[rpc(name = "chain_getMetadataSeq")]
    fn get_metadata_seq(&self, block_number: Option<u64>) -> Result<Option<u64>>;

    /// Return the valid block authors
    #[rpc(name = "chain_getPossibleAuthors")]
    fn get_possible_authors(&self, block_number: Option<u64>) -> Result<Option<Vec<PlatformAddress>>>;

    /// Return the valid block authors
    #[rpc(name = "chain_getValidatorSet")]
    fn get_validator_set(&self, block_number: Option<u64>) -> Result<Option<ValidatorSet>>;
}
