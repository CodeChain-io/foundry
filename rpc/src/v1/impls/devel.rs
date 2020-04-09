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
use super::super::traits::Devel;
use super::super::types::TPSTestSetting;
use ccore::{
    BlockChainClient, BlockId, BlockProducer, DatabaseClient, EngineClient, EngineInfo, MinerService, SnapshotClient,
    TermInfo, COL_STATE,
};
use cjson::bytes::Bytes;
use cnetwork::{unbounded_event_callback, EventSender, IntoSocketAddr};
use csync::BlockSyncEvent;
use ctypes::BlockHash;
use jsonrpc_core::Result;
use kvdb::KeyValueDB;
use primitives::H256;
use rlp::Rlp;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::vec::Vec;

pub struct DevelClient<C, M> {
    client: Arc<C>,
    db: Arc<dyn KeyValueDB>,
    miner: Arc<M>,
    block_sync: Option<EventSender<BlockSyncEvent>>,
}

impl<C, M> DevelClient<C, M>
where
    C: DatabaseClient,
{
    pub fn new(client: Arc<C>, miner: Arc<M>, block_sync: Option<EventSender<BlockSyncEvent>>) -> Self {
        let db = client.database();
        Self {
            client,
            db,
            miner,
            block_sync,
        }
    }
}

impl<C, M> Devel for DevelClient<C, M>
where
    C: BlockChainClient
        + BlockProducer
        + DatabaseClient
        + EngineInfo
        + EngineClient
        + TermInfo
        + SnapshotClient
        + 'static,
    M: MinerService + 'static,
{
    fn get_state_trie_keys(&self, offset: usize, limit: usize) -> Result<Vec<H256>> {
        let iter = self.db.iter(COL_STATE);
        Ok(iter.skip(offset).take(limit).map(|val| H256::from(val.0.deref())).collect())
    }

    fn get_state_trie_value(&self, key: H256) -> Result<Vec<Bytes>> {
        match self.db.get(COL_STATE, &key).map_err(errors::core)? {
            Some(value) => {
                let rlp = Rlp::new(&value);
                Ok(rlp.as_list::<Vec<u8>>().map_err(|e| errors::rlp(&e))?.into_iter().map(Bytes::from).collect())
            }
            None => Ok(Vec::new()),
        }
    }

    fn start_sealing(&self) -> Result<()> {
        self.miner.start_sealing(&*self.client);
        Ok(())
    }

    fn stop_sealing(&self) -> Result<()> {
        self.miner.stop_sealing();
        Ok(())
    }

    fn get_block_sync_peers(&self) -> Result<Vec<SocketAddr>> {
        if let Some(block_sync) = self.block_sync.as_ref() {
            let (sender, receiver) = unbounded_event_callback();
            block_sync.send(BlockSyncEvent::GetPeers(sender)).unwrap();
            Ok(receiver.iter().map(|node_id| node_id.into_addr().into()).collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn get_peer_best_block_hashes(&self) -> Result<Vec<(SocketAddr, BlockHash)>> {
        if let Some(block_sync) = self.block_sync.as_ref() {
            let (sender, receiver) = unbounded_event_callback();
            block_sync.send(BlockSyncEvent::GetPeerBestBlockHashes(sender)).unwrap();
            Ok(receiver.iter().collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn get_target_block_hashes(&self) -> Result<Vec<BlockHash>> {
        if let Some(block_sync) = self.block_sync.as_ref() {
            let (sender, receiver) = unbounded_event_callback();
            block_sync.send(BlockSyncEvent::GetTargetBlockHashes(sender)).unwrap();
            Ok(receiver.iter().collect())
        } else {
            Ok(Vec::new())
        }
    }

    fn snapshot(&self, block_hash: BlockHash) -> Result<()> {
        self.client.notify_snapshot(BlockId::Hash(block_hash));
        Ok(())
    }

    fn test_tps(&self, _setting: TPSTestSetting) -> Result<f64> {
        unimplemented!()
    }
}
