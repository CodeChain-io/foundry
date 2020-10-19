// Copyright 2020 Kodebox, Inc.
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

use crate::scheme::seal::{Generic as GenericSeal, Seal};
use crate::Error;
use ccrypto::BLAKE_NULL_RLP;
use cdb::HashDB;
use ckey::{Ed25519Public as Public, PlatformAddress};
use coordinator::engine::Initializer;
use cstate::{Metadata, NextValidatorSet, StateDB, StateWithCache, TopLevelState, TopState};
use ctypes::{BlockHash, Header};
use primitives::{Bytes, H256};
use rlp::{Rlp, RlpStream};

pub struct Genesis {
    parent_hash: BlockHash,
    /// The genesis block's author field.
    author: Public,
    /// The genesis block's timestamp field.
    timestamp: u64,
    /// Transactions root of the genesis block. Should be BLAKE_NULL_RLP.
    transactions_root: H256,
    /// The genesis block's extra data field.
    extra_data: Bytes,
    /// Each seal field, expressed as RLP, concatenated.
    seal_rlp: Bytes,

    state_root: H256,
}

impl Genesis {
    // get parameters
    pub fn new(s: coordinator::app_desc::Genesis, coordinator: &impl Initializer) -> Self {
        let seal: Seal = From::from(s.seal);
        let GenericSeal(seal_rlp) = seal.into();
        let db = StateDB::new_with_memorydb();
        let (_, state_root) = Self::initialize_state(db, coordinator).expect("DB error while creating genesis block");
        Genesis {
            parent_hash: s.parent_hash.map_or_else(H256::zero, Into::into).into(),
            author: s.author.map_or_else(Public::default, PlatformAddress::into_pubkey),
            timestamp: s.timestamp.map_or(0, Into::into),
            transactions_root: s.transactions_root.map_or_else(|| BLAKE_NULL_RLP, Into::into),
            extra_data: s.extra_data.map_or_else(Vec::new, Into::into),
            seal_rlp,
            state_root,
        }
    }

    pub fn initialize_state(db: StateDB, coordinator: &impl Initializer) -> Result<(StateDB, H256), Error> {
        let root = BLAKE_NULL_RLP;
        let mut state = TopLevelState::from_existing(db, root)?;

        for _ in 0..coordinator.number_of_sub_storages() {
            state.create_module()?;
        }

        let (validators, chain_params) = coordinator.initialize_chain(&mut state);

        let validator_set = NextValidatorSet::from_compact_validator_set(validators);
        validator_set.save_to_state(&mut state)?;

        *state.get_metadata_mut().unwrap() = Metadata::new(chain_params);

        Ok(state.commit_and_clone_db()?)
    }

    pub fn check_genesis_root(&self, db: &dyn HashDB) -> bool {
        if db.is_empty() {
            return true
        }
        db.contains(&self.state_root)
    }

    pub fn header(&self) -> Header {
        let mut header: Header = Default::default();
        header.set_parent_hash(self.parent_hash);
        header.set_timestamp(self.timestamp);
        header.set_number(0);
        header.set_author(self.author);
        header.set_transactions_root(self.transactions_root);
        header.set_extra_data(self.extra_data.clone());
        header.set_state_root(self.state_root);
        header.set_next_validator_set_hash(BLAKE_NULL_RLP /* This will be calculated from state after https://github.com/CodeChain-io/foundry/issues/142*/);
        header.set_seal({
            let r = Rlp::new(&self.seal_rlp);
            r.iter().map(|f| f.as_raw().to_vec()).collect()
        });
        ctrace!(SPEC, "Genesis header is {:?}", header);
        ctrace!(SPEC, "Genesis header hash is {}", header.hash());
        header
    }

    pub fn block(&self) -> Bytes {
        let empty_list = RlpStream::new_list(0).out();
        let header = self.header();
        let mut ret = RlpStream::new_list(3);
        ret.append(&header);
        ret.append_raw(&empty_list, 1); // evidences
        ret.append_raw(&empty_list, 1);
        ret.out()
    }
}
