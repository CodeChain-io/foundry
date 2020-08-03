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

use crate::account::AccountManager;
pub use ckey::Ed25519Public as Public;
use coordinator::module::*;
use coordinator::{Transaction, TransactionWithMetadata};
use foundry_module_rt::UserModule;
use parking_lot::RwLock;
use remote_trait_object::raw_exchange::{import_service_from_handle, HandleToExchange, Skeleton};
use remote_trait_object::{service, Context as RtoContext, Service};
use std::collections::HashMap;
use std::sync::Arc;

#[service]
pub trait GetAccountAndSeq: Service {
    fn get_account_and_seq(&self, tx: &Transaction) -> Result<(Public, u64), ()>;
}

struct Context {
    account_manager: Option<Box<dyn AccountManager>>,
    get_account_and_seqs: HashMap<String, Box<dyn GetAccountAndSeq>>,
}

impl Service for Context {}

impl Context {
    fn account_manager(&self) -> &dyn AccountManager {
        self.account_manager.as_ref().unwrap().as_ref()
    }

    fn account_and_seq_from_tx(&self, tx: &TransactionWithMetadata) -> Option<(Public, u64)> {
        let get_account_and_seq: &dyn GetAccountAndSeq = match self.get_account_and_seqs.get(tx.tx.tx_type()) {
            Some(get_account_and_seq) => get_account_and_seq.as_ref(),
            None => return None,
        };

        match get_account_and_seq.get_account_and_seq(&tx.tx) {
            Ok((public, seq)) => Some((public, seq)),
            _ => None,
        }
    }
}

impl TxSorter for Context {
    // TODO: Consider origin
    fn sort_txs(&self, txs: &[TransactionWithMetadata]) -> SortedTxs {
        // TODO: Avoid Public hashmap
        let mut accounts: HashMap<Public, Vec<(u64, usize)>> = HashMap::new();
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
            let mut seq_in_state = if let Ok(seq) = self.account_manager().get_sequence(account, true) {
                seq
            } else {
                let tx_indices: Vec<usize> = valid.iter().map(|(_, index)| *index).collect();
                invalid.extend_from_slice(&tx_indices);
                continue
            };
            if valid[0].0 != seq_in_state {
                let tx_indices: Vec<usize> = valid.iter().map(|(_, index)| *index).collect();
                invalid.extend_from_slice(&tx_indices);
            }

            for (seq, index) in valid {
                if *seq != seq_in_state {
                    invalid.push(*index);
                } else {
                    seq_in_state += 1;
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

pub struct Module {
    ctx: Arc<RwLock<Context>>,
}

impl UserModule for Module {
    fn new(_arg: &[u8]) -> Self {
        Module {
            ctx: Arc::new(RwLock::new(Context {
                account_manager: None,
                get_account_and_seqs: Default::default(),
            })),
        }
    }

    fn prepare_service_to_export(&mut self, ctor_name: &str, ctor_arg: &[u8]) -> Skeleton {
        match ctor_name {
            "tx_sorter" => {
                let arg: String = serde_cbor::from_slice(ctor_arg).unwrap();
                assert_eq!(arg, "unused");
                Skeleton::new(Arc::clone(&self.ctx) as Arc<RwLock<dyn TxSorter>>)
            }
            _ => panic!("Unsupported ctor_name in prepare_service_to_export() : {}", ctor_name),
        }
    }

    fn import_service(&mut self, rto_context: &RtoContext, name: &str, handle: HandleToExchange) {
        let entries: Vec<&str> = name.split('/').collect();

        if entries.len() == 1 {
            match name {
                "account_manager" => {
                    self.ctx.write().account_manager.replace(import_service_from_handle(rto_context, handle));
                }
                _ => panic!("Invalid name in import_service()"),
            }
        } else if entries.len() == 2 {
            match entries[1] {
                "get_account_and_seq" => assert!(
                    self.ctx
                        .write()
                        .get_account_and_seqs
                        .insert(entries[0].to_owned(), import_service_from_handle(rto_context, handle))
                        .is_none(),
                    "Duplicate transaction service"
                ),
                _ => panic!("Invalid name in import_service()"),
            }
        } else {
            panic!("Invalid name in import_service()")
        }
    }

    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        unimplemented!()
    }
}
