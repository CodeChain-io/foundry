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

use super::super::errors;
use super::super::traits::Chain;
use super::super::types::{Block, BlockNumberAndHash, Transaction};
use ccore::{AccountData, BlockId, EngineInfo, MiningBlockChainClient, Shard, TermInfo};
use cjson::scheme::Params;
use cjson::uint::Uint;
use ckey::{NetworkId, PlatformAddress};
use cstate::FindDoubleVoteHandler;
use ctypes::{BlockHash, BlockNumber, TxHash};
use jsonrpc_core::Result;
use std::sync::Arc;

pub struct ChainClient<C>
where
    C: MiningBlockChainClient + Shard + EngineInfo, {
    client: Arc<C>,
}

impl<C> ChainClient<C>
where
    C: MiningBlockChainClient + Shard + AccountData + EngineInfo,
{
    pub fn new(client: Arc<C>) -> Self {
        ChainClient {
            client,
        }
    }
}

impl<C> Chain for ChainClient<C>
where
    C: MiningBlockChainClient + Shard + AccountData + EngineInfo + FindDoubleVoteHandler + TermInfo + 'static,
{
    fn get_transaction(&self, transaction_hash: TxHash) -> Result<Option<Transaction>> {
        let id = transaction_hash.into();
        Ok(self.client.transaction(&id).map(From::from))
    }

    fn get_transaction_signer(&self, transaction_hash: TxHash) -> Result<Option<PlatformAddress>> {
        let id = transaction_hash.into();
        Ok(self
            .client
            .transaction(&id)
            .map(|mut tx| PlatformAddress::new_v1(tx.unverified_tx().transaction().network_id, tx.signer())))
    }

    fn contains_transaction(&self, transaction_hash: TxHash) -> Result<bool> {
        Ok(self.client.transaction_block(&transaction_hash.into()).is_some())
    }

    fn get_seq(&self, address: PlatformAddress, block_number: Option<u64>) -> Result<Option<u64>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        let pubkey = address.try_pubkey().map_err(errors::core)?;
        Ok(self.client.seq(pubkey, block_id))
    }

    fn get_balance(&self, aaddress: PlatformAddress, block_number: Option<u64>) -> Result<Option<Uint>> {
        let block_id = block_number.map(BlockId::Number).unwrap_or(BlockId::Latest);
        let pubkey = aaddress.try_pubkey().map_err(errors::core)?;
        Ok(self.client.balance(pubkey, block_id.into()).map(Into::into))
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
        Ok(self.client.block(&id).map(|block| Block::from_core(block.decode(), self.client.network_id())))
    }

    fn get_block_by_hash(&self, block_hash: BlockHash) -> Result<Option<Block>> {
        let id = BlockId::Hash(block_hash);
        Ok(self.client.block(&id).map(|block| {
            let block = block.decode();
            Block::from_core(block, self.client.network_id())
        }))
    }

    fn get_block_transaction_count_by_hash(&self, block_hash: BlockHash) -> Result<Option<usize>> {
        Ok(self.client.block(&BlockId::Hash(block_hash)).map(|block| block.transactions_count()))
    }

    fn get_network_id(&self) -> Result<NetworkId> {
        Ok(self.client.network_id())
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
}
