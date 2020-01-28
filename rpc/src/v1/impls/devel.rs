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
    BlockId, DatabaseClient, EngineClient, EngineInfo, MinerService, MiningBlockChainClient, SignedTransaction,
    SnapshotClient, TermInfo, COL_STATE,
};
use cjson::bytes::Bytes;
use ckey::{Address, KeyPair, Private};
use cnetwork::{unbounded_event_callback, EventSender, IntoSocketAddr};
use csync::BlockSyncEvent;
use ctypes::transaction::{Action, Transaction};
use ctypes::BlockHash;
use jsonrpc_core::Result;
use kvdb::KeyValueDB;
use primitives::H256;
use rlp::Rlp;
use std::net::SocketAddr;
use std::ops::Deref;
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use std::vec::Vec;
use time::PreciseTime;

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
    C: DatabaseClient + EngineInfo + EngineClient + MiningBlockChainClient + TermInfo + SnapshotClient + 'static,
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

    fn test_tps(&self, setting: TPSTestSetting) -> Result<f64> {
        let common_params = self.client.common_params(BlockId::Latest).unwrap();
        let pay_fee = common_params.min_pay_transaction_cost();
        let network_id = common_params.network_id();

        // NOTE: Assuming solo network
        let genesis_secret: Private = "ede1d4ccb4ec9a8bbbae9a13db3f4a7b56ea04189be86ac3a6a439d9a0a1addd".into();
        let genesis_keypair = KeyPair::from_private(genesis_secret).map_err(errors::transaction_core)?;

        let base_seq = self.client.seq(&genesis_keypair.address(), BlockId::Latest).unwrap();

        // Helper macros
        macro_rules! pay_tx {
            ($seq:expr, $address:expr) => {
                pay_tx!($seq, $address, 1)
            };
            ($seq:expr, $address:expr, $quantity: expr) => {
                Transaction {
                    seq: $seq,
                    fee: pay_fee,
                    network_id,
                    action: Action::Pay {
                        receiver: $address,
                        quantity: $quantity,
                    },
                }
            };
        }

        // Helper functions
        fn sign_tx(tx: Transaction, key_pair: &KeyPair) -> SignedTransaction {
            SignedTransaction::new_with_sign(tx, key_pair.private())
        }

        fn tps(count: u64, start_time: PreciseTime, end_time: PreciseTime) -> f64 {
            f64::from(count as u32) * 1000.0_f64 / f64::from(start_time.to(end_time).num_milliseconds() as i32)
        }

        // Main
        let count = setting.count;
        if count == 0 {
            return Ok(0.0)
        }
        let transactions = {
            let mut transactions = Vec::with_capacity(count as usize);
            for i in 0..count {
                let address = Address::random();
                let tx = sign_tx(pay_tx!(base_seq + i, address), &genesis_keypair);
                transactions.push(tx);
            }
            transactions
        };

        let last_hash = transactions.last().unwrap().hash();

        let first_transaction = transactions[0].clone();
        for tx in transactions.into_iter().skip(1) {
            self.client.queue_own_transaction(tx).map_err(errors::transaction_core)?;
        }
        let start_time = PreciseTime::now();
        self.client.queue_own_transaction(first_transaction).map_err(errors::transaction_core)?;
        while !self.client.is_pending_queue_empty() {
            thread::sleep(Duration::from_millis(50));
        }
        while self.client.transaction(&last_hash.into()).is_none() {}

        let end_time = PreciseTime::now();
        Ok(tps(count, start_time, end_time))
    }
}
