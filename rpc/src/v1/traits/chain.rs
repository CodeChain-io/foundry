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

use super::super::types::{Block, BlockNumberAndHash, Transaction, UnsignedTransaction};
use cjson::scheme::Params;
use cjson::uint::Uint;
use ckey::{NetworkId, PlatformAddress};
use ctypes::{BlockHash, BlockNumber, ShardId, TxHash};
use jsonrpc_core::Result;
use primitives::H256;

#[rpc(server)]
pub trait Chain {
    /// Gets transaction with given hash.
    #[rpc(name = "chain_getTransaction")]
    fn get_transaction(&self, transaction_hash: TxHash) -> Result<Option<Transaction>>;

    /// Gets the signer of transaction with given hash.
    #[rpc(name = "chain_getTransactionSigner")]
    fn get_transaction_signer(&self, transaction_hash: TxHash) -> Result<Option<PlatformAddress>>;

    /// Query whether the chain has the transaction with given transaction hash.
    #[rpc(name = "chain_containsTransaction")]
    fn contains_transaction(&self, transaction_hash: TxHash) -> Result<bool>;

    #[rpc(name = "chain_containTransaction")]
    fn contain_transaction(&self, transaction_hash: TxHash) -> Result<bool>;

    /// Gets seq with given account.
    #[rpc(name = "chain_getSeq")]
    fn get_seq(&self, address: PlatformAddress, block_number: Option<u64>) -> Result<Option<u64>>;

    /// Gets balance with given account.
    #[rpc(name = "chain_getBalance")]
    fn get_balance(&self, address: PlatformAddress, block_number: Option<u64>) -> Result<Option<Uint>>;

    /// Gets the genesis accounts
    #[rpc(name = "chain_getGenesisAccounts")]
    fn get_genesis_accounts(&self) -> Result<Vec<PlatformAddress>>;

    /// Gets the number of shards
    #[rpc(name = "chain_getNumberOfShards")]
    fn get_number_of_shards(&self, block_number: Option<u64>) -> Result<Option<ShardId>>;

    /// Gets shard id
    #[rpc(name = "chain_getShardIdByHash")]
    fn get_shard_id_by_hash(&self, create_shard_tx_hash: TxHash, block_number: Option<u64>) -> Result<Option<ShardId>>;

    /// Gets shard root
    #[rpc(name = "chain_getShardRoot")]
    fn get_shard_root(&self, shard_id: ShardId, block_number: Option<u64>) -> Result<Option<H256>>;

    /// Gets shard owners
    #[rpc(name = "chain_getShardOwners")]
    fn get_shard_owners(&self, shard_id: ShardId, block_number: Option<u64>) -> Result<Option<Vec<PlatformAddress>>>;

    /// Gets shard users
    #[rpc(name = "chain_getShardUsers")]
    fn get_shard_users(&self, shard_id: ShardId, block_number: Option<u64>) -> Result<Option<Vec<PlatformAddress>>>;

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

    ///Gets the minimum transaction fee of the given name.
    #[rpc(name = "chain_getMinTransactionFee")]
    fn get_min_transaction_fee(&self, action_type: String, block_number: Option<u64>) -> Result<Option<u64>>;

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

    /// Execute Transactions
    #[rpc(name = "chain_executeTransaction")]
    fn execute_transaction(&self, tx: UnsignedTransaction, sender: PlatformAddress) -> Result<Option<String>>;
}
