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

use super::seal::Generic as GenericSeal;
use super::Genesis;
use crate::consensus::{ConsensusEngine, NullEngine, Solo, Tendermint};
use crate::error::Error;
use ccrypto::BLAKE_NULL_RLP;
use cdb::HashDB;
use ckey::Address;
use ctypes::{BlockHash, Header};
use parking_lot::RwLock;
use primitives::{Bytes, H256};
use rlp::{Rlp, RlpStream};
use std::io::Read;
use std::sync::Arc;

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Scheme {
    /// User friendly scheme name
    pub name: String,
    /// What engine are we using for this?
    pub engine: Arc<dyn ConsensusEngine>,
    /// Name of the subdir inside the main data dir to use for chain data and settings.
    pub data_dir: String,

    /// Known nodes on the network in enode format.
    pub nodes: Vec<String>,

    /// The genesis block's parent hash field.
    pub parent_hash: BlockHash,
    /// The genesis block's author field.
    pub author: Address,
    /// The genesis block's timestamp field.
    pub timestamp: u64,
    /// Transactions root of the genesis block. Should be BLAKE_NULL_RLP.
    pub transactions_root: H256,
    /// The genesis block's extra data field.
    pub extra_data: Bytes,
    /// Each seal field, expressed as RLP, concatenated.
    pub seal_rlp: Bytes,

    /// May be prepopulated if we know this in advance.
    state_root_memo: RwLock<H256>,

    /// Application initial state
    pub app_state: String,
}

// helper for formatting errors.
fn fmt_err<F: ::std::fmt::Display>(f: F) -> String {
    format!("Scheme json is invalid: {}", f)
}

macro_rules! load_bundled {
    ($e:expr) => {
        Scheme::load(include_bytes!(concat!("../../res/", $e, ".json")) as &[u8]).expect(concat!(
            "Chain scheme ",
            $e,
            " is invalid."
        ))
    };
}

impl Scheme {
    /// Convert engine scheme into a arc'd Engine of the right underlying type.
    /// TODO avoid this hard-coded nastiness - use dynamic-linked plugin framework instead.
    fn engine(engine_scheme: cjson::scheme::Engine) -> Arc<dyn ConsensusEngine> {
        match engine_scheme {
            cjson::scheme::Engine::Null => Arc::new(NullEngine::default()),
            cjson::scheme::Engine::Solo(_solo) => Arc::new(Solo::new()),
            cjson::scheme::Engine::Tendermint(tendermint) => Tendermint::new(tendermint.params.into()),
        }
    }

    pub fn check_genesis_root(&self, db: &dyn HashDB) -> bool {
        if db.is_empty() {
            return true
        }
        db.contains(&self.state_root())
    }

    /// Return the state root for the genesis state, memoising accordingly.
    pub fn state_root(&self) -> H256 {
        *self.state_root_memo.read()
    }

    /// Loads scheme from json file. Provide factories for executing contracts and ensuring
    /// storage goes to the right place.
    pub fn load<R>(reader: R) -> Result<Self, String>
    where
        R: Read, {
        cjson::scheme::Scheme::load(reader).map_err(fmt_err).and_then(|x| load_from(x).map_err(fmt_err))
    }

    /// Create a new test Scheme.
    pub fn new_test() -> Self {
        load_bundled!("null")
    }

    /// Create a new Scheme with Solo consensus which does internal sealing (not requiring
    /// work).
    pub fn new_test_solo() -> Self {
        load_bundled!("solo")
    }

    /// Create a new Scheme with Tendermint consensus which does internal sealing (not requiring
    /// work).
    pub fn new_test_tendermint() -> Self {
        load_bundled!("tendermint")
    }

    /// Get the header of the genesis block.
    pub fn genesis_header(&self) -> Header {
        let mut header: Header = Default::default();
        header.set_parent_hash(self.parent_hash);
        header.set_timestamp(self.timestamp);
        header.set_number(0);
        header.set_author(self.author);
        header.set_transactions_root(self.transactions_root);
        header.set_extra_data(self.extra_data.clone());
        header.set_state_root(self.state_root());
        header.set_next_validator_set_hash(BLAKE_NULL_RLP /* This will be calculated from state after https://github.com/CodeChain-io/foundry/issues/142*/);
        header.set_seal({
            let r = Rlp::new(&self.seal_rlp);
            r.iter().map(|f| f.as_raw().to_vec()).collect()
        });
        ctrace!(SPEC, "Genesis header is {:?}", header);
        ctrace!(SPEC, "Genesis header hash is {}", header.hash());
        header
    }

    /// Compose the genesis block for this chain.
    pub fn genesis_block(&self) -> Bytes {
        let empty_list = RlpStream::new_list(0).out();
        let header = self.genesis_header();
        let mut ret = RlpStream::new_list(2);
        ret.append(&header);
        ret.append_raw(&empty_list, 1);
        ret.out()
    }
}

/// Load from JSON object.
fn load_from(s: cjson::scheme::Scheme) -> Result<Scheme, Error> {
    let g = Genesis::from(s.genesis);
    let GenericSeal(seal_rlp) = g.seal.into();
    let engine = Scheme::engine(s.engine);

    let mut s = Scheme {
        name: s.name.clone(),
        engine,
        data_dir: s.data_dir.unwrap_or(s.name),
        nodes: s.nodes.unwrap_or_else(Vec::new),
        parent_hash: g.parent_hash,
        transactions_root: g.transactions_root,
        author: g.author,
        timestamp: g.timestamp,
        extra_data: g.extra_data,
        seal_rlp,
        state_root_memo: RwLock::new(Default::default()), // will be overwritten right after.

        app_state: s.app_state,
    };

    // use memoized state root if provided.
    if let Some(root) = g.state_root {
        *s.state_root_memo.get_mut() = root;
    }

    Ok(s)
}
