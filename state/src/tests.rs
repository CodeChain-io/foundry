// Copyright 2018, 2020 Kodebox, Inc.
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

pub mod helpers {
    use crate::impls::TopLevelState;
    use crate::{FindDoubleVoteHandler, Metadata, MetadataAddress, StateDB};
    use cdb::AsHashDB;
    use ctypes::CommonParams;
    use kvdb::KeyValueDB;
    use merkle_trie::{TrieFactory, TrieMut};
    use primitives::H256;
    use rlp::Encodable;
    use std::sync::Arc;
    pub struct TestClient {}

    impl FindDoubleVoteHandler for TestClient {}

    pub fn get_memory_db() -> Arc<dyn KeyValueDB> {
        Arc::new(kvdb_memorydb::create(1))
    }

    pub fn get_temp_state_db() -> StateDB {
        StateDB::new_with_memorydb()
    }

    pub fn get_temp_state() -> TopLevelState {
        get_temp_state_with_metadata(CommonParams::default_for_test())
    }

    pub fn get_temp_state_with_metadata(params: CommonParams) -> TopLevelState {
        let state_db = get_temp_state_db();
        empty_top_state_with_metadata(state_db, params)
    }

    pub fn get_test_client() -> TestClient {
        TestClient {}
    }

    /// Creates new state with empty state root
    /// Used for tests.
    pub fn empty_top_state(mut db: StateDB) -> TopLevelState {
        let mut root = H256::new();
        // init trie and reset root too null
        let _ = TrieFactory::create(db.as_hashdb_mut(), &mut root);

        TopLevelState::from_existing(db, root).expect("The empty trie root was initialized")
    }

    /// Creates new state with empty state root
    /// Used for tests.
    pub fn empty_top_state_with_metadata(mut db: StateDB, params: CommonParams) -> TopLevelState {
        let mut root = H256::new();
        // init trie and reset root too null
        {
            let mut t = TrieFactory::create(db.as_hashdb_mut(), &mut root);
            t.insert(&*MetadataAddress::new(), &Metadata::new(0, params).rlp_bytes()).unwrap();
        }

        TopLevelState::from_existing(db, root).expect("The empty trie root was initialized")
    }
}
