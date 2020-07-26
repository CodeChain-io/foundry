// Copyright 2019-2020 Kodebox, Inc.
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

use super::super::errors;
use super::super::traits::Mempool;
use super::super::types::PendingTransactions;
use ccore::{BlockChainClient, EngineInfo, UnverifiedTransaction, VerifiedTransaction};
use cjson::bytes::Bytes;
use ctypes::TxHash;
use jsonrpc_core::Result;
use rlp::Rlp;
use std::convert::TryInto;
use std::sync::Arc;

pub struct MempoolClient<C> {
    client: Arc<C>,
}

impl<C> MempoolClient<C> {
    pub fn new(client: Arc<C>) -> Self {
        MempoolClient {
            client,
        }
    }
}

impl<C> Mempool for MempoolClient<C>
where
    C: BlockChainClient + EngineInfo + 'static,
{
    fn send_signed_transaction(&self, raw: Bytes) -> Result<TxHash> {
        Rlp::new(&raw.into_vec())
            .as_val()
            .map_err(|e| errors::rlp(&e))
            .and_then(|tx: UnverifiedTransaction| tx.try_into().map_err(errors::transaction_core))
            .and_then(|signed: VerifiedTransaction| {
                let hash = signed.hash();
                match self.client.queue_own_transaction(signed) {
                    Ok(_) => Ok(hash),
                    Err(e) => Err(errors::transaction_core(e)),
                }
            })
            .map(Into::into)
    }

    fn delete_all_pending_transactions(&self) -> Result<()> {
        self.client.delete_all_pending_transactions();
        Ok(())
    }

    fn get_pending_transactions(&self, from: Option<u64>, to: Option<u64>) -> Result<PendingTransactions> {
        Ok(self.client.ready_transactions(from.unwrap_or(0)..to.unwrap_or(u64::MAX)).into())
    }

    fn get_pending_transactions_count(&self, from: Option<u64>, to: Option<u64>) -> Result<usize> {
        Ok(self.client.count_pending_transactions(from.unwrap_or(0)..to.unwrap_or(u64::MAX)))
    }
}
