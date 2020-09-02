// Copyright 2020 Kodebox, Inc.
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

use super::GetAccountAndSeq;
use super::ServiceHandler;
use crate::common::*;
pub use ckey::{Ed25519Private as Private, Ed25519Public as Public};
use coordinator::module::*;
use coordinator::TransactionWithMetadata;
use std::collections::HashMap;

impl ServiceHandler {
    fn account_and_seq_from_tx(&self, tx: &TransactionWithMetadata) -> Option<(Public, TxSeq)> {
        let guard = self.get_account_and_seqs.read();
        let get_account_and_seq: &dyn GetAccountAndSeq = match guard.get(tx.tx.tx_type()) {
            Some(get_account_and_seq) => get_account_and_seq.as_ref(),
            None => return None,
        };

        match get_account_and_seq.get_account_and_seq(&tx.tx) {
            Ok((public, seq)) => Some((public, seq)),
            _ => None,
        }
    }
}

impl TxSorter for ServiceHandler {
    // TODO: Consider origin
    fn sort_txs(&self, session: SessionId, txs: &[TransactionWithMetadata]) -> SortedTxs {
        // TODO: Avoid Public hashmap
        let mut accounts: HashMap<Public, Vec<(TxSeq, usize)>> = HashMap::new();
        let mut invalid: Vec<usize> = Vec::new();

        for (i, tx) in txs.iter().enumerate() {
            if let Some((public, seq)) = self.account_and_seq_from_tx(tx) {
                if let Some(valid) = accounts.get_mut(&public) {
                    valid.push((seq, i));
                } else {
                    accounts.insert(public, vec![(seq, i)]);
                }
            } else {
                invalid.push(i);
            }
        }

        let mut sorted: Vec<usize> = Vec::new();

        for (account, valid) in accounts.iter_mut() {
            valid.sort();
            let seq_in_state = if let Ok(account) = self.account_manager.read().get_account(session, account, true) {
                account.seq
            } else {
                let tx_indices: Vec<usize> = valid.iter().map(|(_, index)| *index).collect();
                invalid.extend_from_slice(&tx_indices);
                continue
            };

            for (seq, index) in valid {
                if *seq < seq_in_state {
                    invalid.push(*index);
                } else {
                    sorted.push(*index);
                }
            }
        }
        SortedTxs {
            sorted,
            invalid,
        }
    }
}
