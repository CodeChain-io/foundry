// Copyright 2018-2019 Kodebox, Inc.
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

use std::convert::TryInto;
use std::sync::Arc;

use ccore::{AccountData, BlockId, EngineInfo, ExecuteClient, MiningBlockChainClient, TermInfo, TextClient};
use cjson::scheme::Params;
use cjson::uint::Uint;
use ckey::{public_to_address, NetworkId, PlatformAddress, Public};
use cstate::FindActionHandler;
use ctypes::transaction::{Action, ShardTransaction as ShardTransactionType};
use ctypes::{BlockHash, BlockNumber, Tracker, TxHash};
use primitives::Bytes as BytesArray;

use jsonrpc_core::Result;

use super::super::errors;
use super::super::traits::Chain;
use super::super::types::{Block, BlockNumberAndHash, Text, Transaction, UnsignedTransaction};

pub struct ChainClient<C>
where
    C: MiningBlockChainClient + ExecuteClient + EngineInfo, {
    client: Arc<C>,
}

impl<C> ChainClient<C>
where
    C: MiningBlockChainClient + AccountData + ExecuteClient + EngineInfo + TextClient,
{
    pub fn new(client: Arc<C>) -> Self {
        ChainClient {
            client,
        }
    }
}

impl<C> Chain for ChainClient<C>
where
    C: MiningBlockChainClient
        + AccountData
        + ExecuteClient
        + EngineInfo
        + FindActionHandler
        + TextClient
        + TermInfo
        + 'static,
{
    fn get_transaction(&self, transaction_hash: TxHash) -> Result<Option<Transaction>> {
        let id = transaction_hash.into();
        Ok(self.client.transaction(&id).map(From::from))
    }

    fn get_transaction_signer(&self, transaction_hash: TxHash) -> Result<Option<PlatformAddress>> {
        let id = transaction_hash.into();
        Ok(self.client.transaction(&id).map(|mut tx| {
            let address = public_to_address(&tx.signer());
            PlatformAddress::new_v1(tx.network_id, address)
        }))
    }

    fn contains_transaction(&self, transaction_hash: TxHash) -> Result<bool> {
        Ok(self.client.transaction_block(&transaction_hash.into()).is_some())
    }

    fn contain_transaction(&self, transaction_hash: TxHash) -> Result<bool> {
        self.contains_transaction(transaction_hash)
    }

    fn get_transaction_by_tracker(&self, tracker: Tracker) -> Result<Option<Transaction>> {
        Ok(self.client.transaction_by_tracker(&tracker).map(From::from))
    }

    fn get_text(&self, transaction_hash: TxHash, block_number: Option<u64>) -> Result<Option<Text>> {
        if block_number == Some(0) {
            return Ok(None)
        }
        let block_id = block_number.map(BlockId::from).unwrap_or(BlockId::Latest);
        Ok(self.client.get_text(transaction_hash, block_id).map_err(errors::transaction_state)?.map(|text| {
            let parent_block_id = block_number.map(|n| (n - 1).into()).unwrap_or(BlockId::ParentOfLatest);
            Text::from_core(text, self.client.common_params(parent_block_id).unwrap().network_id())
        }))
    }

    fn get_seq(&self, address: PlatformAddress, block_number: Option<u64>) -> Result<Option<u64>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        let address = address.try_address().map_err(errors::core)?;
        Ok(self.client.seq(address, block_id))
    }

    fn get_balance(&self, aaddress: PlatformAddress, block_number: Option<u64>) -> Result<Option<Uint>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        let address = aaddress.try_address().map_err(errors::core)?;
        Ok(self.client.balance(address, block_id.into()).map(Into::into))
    }

    fn get_regular_key(&self, address: PlatformAddress, block_number: Option<u64>) -> Result<Option<Public>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        let address = address.try_address().map_err(errors::core)?;
        Ok(self.client.regular_key(address, block_id.into()))
    }

    fn get_regular_key_owner(&self, public: Public, block_number: Option<u64>) -> Result<Option<PlatformAddress>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        Ok(self.client.regular_key_owner(&public_to_address(&public), block_id.into()).and_then(|address| {
            let parent_block_id = block_number.map(|n| (n - 1).into()).unwrap_or(BlockId::ParentOfLatest);
            let network_id = self.client.common_params(parent_block_id).unwrap().network_id();
            Some(PlatformAddress::new_v1(network_id, address))
        }))
    }

    fn get_genesis_accounts(&self) -> Result<Vec<PlatformAddress>> {
        Ok(self.client.genesis_accounts())
    }

    fn get_best_block_number(&self) -> Result<BlockNumber> {
        Ok(self.client.chain_info().best_block_number)
    }

    fn get_best_block_id(&self) -> Result<BlockNumberAndHash> {
        let chain_info = self.client.chain_info();
        Ok(BlockNumberAndHash {
            number: chain_info.best_block_number,
            hash: chain_info.best_block_hash,
        })
    }

    fn get_block_hash(&self, block_number: u64) -> Result<Option<BlockHash>> {
        Ok(self.client.block_hash(&BlockId::Number(block_number)))
    }

    fn get_block_by_number(&self, block_number: u64) -> Result<Option<Block>> {
        let id = BlockId::Number(block_number);
        Ok(self.client.block(&id).map(|block| {
            let block_id_to_read_params = if block_number == 0 {
                0.into()
            } else {
                (block_number - 1).into()
            };
            Block::from_core(block.decode(), self.client.common_params(block_id_to_read_params).unwrap().network_id())
        }))
    }

    fn get_block_by_hash(&self, block_hash: BlockHash) -> Result<Option<Block>> {
        let id = BlockId::Hash(block_hash);
        Ok(self.client.block(&id).map(|block| {
            let block = block.decode();
            let block_id_to_read_params = if block.header.number() == 0 {
                0.into()
            } else {
                (*block.header.parent_hash()).into()
            };
            Block::from_core(block, self.client.common_params(block_id_to_read_params).unwrap().network_id())
        }))
    }

    fn get_block_transaction_count_by_hash(&self, block_hash: BlockHash) -> Result<Option<usize>> {
        Ok(self.client.block(&BlockId::Hash(block_hash)).map(|block| block.transactions_count()))
    }

    fn get_min_transaction_fee(&self, action_type: String, block_number: Option<u64>) -> Result<Option<u64>> {
        if block_number == Some(0) {
            return Ok(None)
        }
        // Unlike other RPCs, use the latest parameters if the block number is `null`.
        let block_id = block_number.map(|n| (n - 1).into()).unwrap_or(BlockId::Latest);
        if let Some(common_parameters) = self.client.common_params(block_id) {
            Ok(match action_type.as_str() {
                "mintAsset" => Some(common_parameters.min_asset_mint_cost()),
                "transferAsset" => Some(common_parameters.min_asset_transfer_cost()),
                "changeAssetScheme" => Some(common_parameters.min_asset_scheme_change_cost()),
                "increaseAssetSupply" => Some(common_parameters.min_asset_supply_increase_cost()),
                "unwrapCCC" => Some(common_parameters.min_asset_unwrap_ccc_cost()),
                "pay" => Some(common_parameters.min_pay_transaction_cost()),
                "setRegularKey" => Some(common_parameters.min_set_regular_key_transaction_cost()),
                "createShard" => Some(common_parameters.min_create_shard_transaction_cost()),
                "setShardOwners" => Some(common_parameters.min_set_shard_owners_transaction_cost()),
                "setShardUsers" => Some(common_parameters.min_set_shard_users_transaction_cost()),
                "wrapCCC" => Some(common_parameters.min_wrap_ccc_transaction_cost()),
                "store" => Some(common_parameters.min_store_transaction_cost()),
                "remove" => Some(common_parameters.min_remove_transaction_cost()),
                "custom" => Some(common_parameters.min_custom_transaction_cost()),

                _ => None,
            })
        } else {
            Ok(None)
        }
    }

    fn get_mining_reward(&self, block_number: u64) -> Result<Option<u64>> {
        Ok(self.client.mining_reward(block_number))
    }

    fn get_network_id(&self) -> Result<NetworkId> {
        Ok(self.client.common_params(BlockId::Latest).unwrap().network_id())
    }

    fn get_common_params(&self, block_number: Option<u64>) -> Result<Option<Params>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        Ok(self.client.common_params(block_id).map(Params::from))
    }

    fn get_term_metadata(&self, block_number: Option<u64>) -> Result<Option<(u64, u64)>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        let last_term_finished_block_num = self.client.last_term_finished_block_num(block_id);
        let current_term_id = self.client.current_term_id(block_id);
        match (last_term_finished_block_num, current_term_id) {
            (Some(last_term_finished_block_num), Some(current_term_id)) => {
                Ok(Some((last_term_finished_block_num, current_term_id)))
            }
            (None, None) => Ok(None),
            _ => unreachable!(),
        }
    }

    fn get_metadata_seq(&self, block_number: Option<u64>) -> Result<Option<u64>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        Ok(self.client.metadata_seq(block_id))
    }

    fn get_possible_authors(&self, block_number: Option<u64>) -> Result<Option<Vec<PlatformAddress>>> {
        Ok(self.client.possible_authors(block_number).map_err(errors::core)?)
    }

    fn execute_vm(
        &self,
        tx: UnsignedTransaction,
        params: Vec<Vec<BytesArray>>,
        indices: Vec<usize>,
    ) -> Result<Vec<String>> {
        let action = tx.action.try_into().map_err(errors::conversion)?;
        if let Action::TransferAsset {
            inputs,
            ..
        } = &action
        {
            let transaction = Option::<ShardTransactionType>::from(action.clone()).unwrap();
            Ok(self.client.execute_vm(&transaction, inputs, &params, &indices).map_err(errors::core)?)
        } else {
            Err(errors::transfer_only())
        }
    }
}
