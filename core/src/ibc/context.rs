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

pub struct TopLevelKVStore<'a> {
    state: &'a mut TopLevelState,
}

impl<'a> TopLevelKVStore<'a> {
    pub fn key(path: &str) -> H256 {
        let mut rlp = RlpStream::new_list(2);
        rlp.append(&"IBCData");
        rlp.append(&path);
        blake256(rlp.drain())
    }
}

impl<'a> KVStore for TopLevelKVStore<'a> {
    fn get(&self, path: Path) -> Option<Bytes> {
        let key = TopLevelKVStore::key(path);
        self.state.ibc_data(&key).expect("Get key").map(Bytes::from)
    }

    fn contains_key(&self, path: Path) -> bool {
        let key = TopLevelKVStore::key(path);
        self.state.ibc_data(&key).expect("Get key").is_some()
    }

    fn insert(&mut self, path: Path, value: &[u8]) -> Option<Bytes> {
        // FIXME: the update_ibc_data returns the previous data.
        // When the previous data is empty, it should return None.
        // Currently it is returning an empty RLP array.
        let prev = self.get(path);
        let key = TopLevelKVStore::key(path);
        self.state.update_ibc_data(&key, value.to_vec()).expect("Set in IBC KVStore");
        prev
    }

    fn remove(&mut self, path: Path) -> Option<Bytes> {
        let prev = self.get(path);
        let key = TopLevelKVStore::key(path);
        self.state.remove_ibc_data(&key);
        prev
    }

    fn root(&self) -> H256 {
        self.state.root()
    }

    fn make_proof(&self, path: Path) -> (CryptoProofUnit, CryptoProof) {
        let key = TopLevelKVStore::key(path);
        self.state.ibc_data_proof(&key).unwrap()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use cstate::tests::helpers::get_temp_state;
    use cstate::StateWithCache;
    use merkle_trie::proof::verify;

    #[test]
    fn test_verify() {
        let data = rlp::encode(&b"data".to_vec());
        let mut state = {
            let mut state = get_temp_state();

            state.update_ibc_data(&TopLevelKVStore::key("0"), data.clone()).unwrap();
            state.commit().unwrap();
            state
        };
        let state_root = state.root();

        let context = TopLevelContext::new(&mut state, 1);
        let (proof_unit, proof) = context.get_kv_store().make_proof("0");
        let read_value = context.get_kv_store().get("0");
        assert_eq!(proof_unit.root, state_root, "Test root");
        assert_eq!(
            rustc_hex::ToHex::to_hex(proof_unit.key.as_slice()),
            rustc_hex::ToHex::to_hex(TopLevelKVStore::key("0").to_vec().as_slice()),
            "Test key"
        );
        assert_eq!(Some(data.clone()), read_value, "Read value");
        assert_eq!(proof_unit.value, Some(data), "Test value");
        assert!(verify(&proof, &proof_unit));
    }
}
