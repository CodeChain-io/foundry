// Copyright 2019-2020 Kodebox, Inc.
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
use super::kv_store::{KVStore, Path};
use ccrypto::blake256;
use cstate::{TopLevelState, TopState, TopStateView};
use merkle_trie::proof::{CryptoProof, CryptoProofUnit};
use primitives::{Bytes, H256};
use rlp::RlpStream;

pub trait Context {
    fn get_current_height(&self) -> u64;
    fn get_kv_store_mut(&mut self) -> &mut dyn KVStore;
    fn get_kv_store(&self) -> &dyn KVStore;
}

pub struct TopLevelContext<'a> {
    kv_store: TopLevelKVStore<'a>,
    current_height: u64,
}

impl<'a> TopLevelContext<'a> {
    pub fn new(state: &'a mut TopLevelState, current_height: u64) -> Self {
        TopLevelContext {
            kv_store: TopLevelKVStore {
                state,
            },
            current_height,
        }
    }
}

impl<'a> Context for TopLevelContext<'a> {
    fn get_kv_store_mut(&mut self) -> &mut dyn KVStore {
        &mut self.kv_store
    }

    fn get_current_height(&self) -> u64 {
        self.current_height
    }

    fn get_kv_store(&self) -> &dyn KVStore {
        &self.kv_store
    }
}

struct TopLevelKVStore<'a> {
    state: &'a mut TopLevelState,
}

impl<'a> TopLevelKVStore<'a> {
    fn key(path: &str) -> H256 {
        let mut rlp = RlpStream::new_list(2);
        rlp.append(&"IBCData");
        rlp.append(&path);
        blake256(rlp.drain())
    }
}

impl<'a> KVStore for TopLevelKVStore<'a> {
    fn get(&self, path: Path) -> Bytes {
        let key = TopLevelKVStore::key(path);
        self.state.ibc_data(&key).expect("Get key").expect("Data empty").into()
    }

    fn has(&self, path: Path) -> bool {
        let key = TopLevelKVStore::key(path);
        self.state.ibc_data(&key).expect("Get key").is_some()
    }

    fn set(&mut self, path: Path, value: &[u8]) {
        let key = TopLevelKVStore::key(path);
        self.state.update_ibc_data(&key, value.to_vec()).expect("Set in IBC KVStore")
    }

    fn delete(&mut self, path: Path) {
        let key = TopLevelKVStore::key(path);
        self.state.remove_ibc_data(&key);
    }

    fn root(&self) -> H256 {
        self.state.root()
    }

    fn make_proof(&self, path: Path) -> (CryptoProofUnit, CryptoProof) {
        let key = TopLevelKVStore::key(path);
        self.state.ibc_data_proof(&key).unwrap()
    }
}
