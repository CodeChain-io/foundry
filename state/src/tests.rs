// Copyright 2018, 2020 Kodebox, Inc.
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

pub mod helpers {
    use crate::impls::TopLevelState;
    use crate::{Metadata, MetadataAddress, StateDB};
    use cdb::AsHashDB;
    use ctypes::ConsensusParams;
    use kvdb::KeyValueDB;
    use merkle_trie::{TrieFactory, TrieMut};
    use primitives::H256;
    use rlp::Encodable;
    use std::sync::Arc;
    pub struct TestClient {}

    pub fn get_memory_db() -> Arc<dyn KeyValueDB> {
        Arc::new(kvdb_memorydb::create(1))
    }

    pub fn get_temp_state_db() -> StateDB {
        StateDB::new_with_memorydb()
    }

    pub fn get_temp_state() -> TopLevelState {
        get_temp_state_with_metadata(ConsensusParams::default_for_test())
    }

    pub fn get_temp_state_with_metadata(consensus_params: ConsensusParams) -> TopLevelState {
        let state_db = get_temp_state_db();
        empty_top_state_with_metadata(state_db, consensus_params)
    }

    pub fn get_test_client() -> TestClient {
        TestClient {}
    }

    /// Creates new state with empty state root
    /// Used for tests.
    pub fn empty_top_state(mut db: StateDB) -> TopLevelState {
        let mut root = H256::default();
        // init trie and reset root too null
        let _ = TrieFactory::create(db.as_hashdb_mut(), &mut root);

        TopLevelState::from_existing(db, root).expect("The empty trie root was initialized")
    }

    /// Creates new state with empty state root
    /// Used for tests.
    pub fn empty_top_state_with_metadata(mut db: StateDB, consensus_params: ConsensusParams) -> TopLevelState {
        let mut root = H256::default();
        // init trie and reset root too null
        {
            let mut t = TrieFactory::create(db.as_hashdb_mut(), &mut root);
            t.insert(MetadataAddress::new().as_ref(), &Metadata::new(consensus_params).rlp_bytes()).unwrap();
        }

        TopLevelState::from_existing(db, root).expect("The empty trie root was initialized")
    }
}
