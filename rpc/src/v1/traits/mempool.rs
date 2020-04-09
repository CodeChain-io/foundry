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

use super::super::types::PendingTransactions;
use cjson::bytes::Bytes;
use ctypes::TxHash;
use jsonrpc_core::Result;

#[rpc(server)]
pub trait Mempool {
    /// Sends signed transaction, returning its hash.
    #[rpc(name = "mempool_sendSignedTransaction")]
    fn send_signed_transaction(&self, raw: Bytes) -> Result<TxHash>;

    /// Deletes all pending transactions in the mem pool, including future queue.
    #[rpc(name = "mempool_deleteAllPendingTransactions")]
    fn delete_all_pending_transactions(&self) -> Result<()>;

    /// Gets transactions in the current mem pool. future_included is set to check whether append future queue or not.
    #[rpc(name = "mempool_getPendingTransactions")]
    fn get_pending_transactions(
        &self,
        from: Option<u64>,
        to: Option<u64>,
        future_included: Option<bool>,
    ) -> Result<PendingTransactions>;

    /// Gets the count of transactions in the current mem pool.
    #[rpc(name = "mempool_getPendingTransactionsCount")]
    fn get_pending_transactions_count(
        &self,
        from: Option<u64>,
        to: Option<u64>,
        future_included: Option<bool>,
    ) -> Result<usize>;
}
