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

use super::pod_state::PodAccounts;
use super::seal::Generic as GenericSeal;
use super::Genesis;
use crate::blockchain::HeaderProvider;
use crate::codechain_machine::CodeChainMachine;
use crate::consensus::{CodeChainEngine, NullEngine, Solo, Tendermint};
use crate::error::{Error, SchemeError};
use ccrypto::{blake256, BLAKE_NULL_RLP};
use cdb::{AsHashDB, HashDB};
use ckey::Address;
use cstate::{Metadata, MetadataAddress, StateDB, StateResult};
use ctypes::errors::SyntaxError;
use ctypes::{BlockHash, CommonParams, Header};
use merkle_trie::{TrieFactory, TrieMut};
use parking_lot::RwLock;
use primitives::{Bytes, H256};
use rlp::{Encodable, Rlp, RlpStream};
use std::io::Read;
use std::sync::Arc;

/// Parameters for a block chain; includes both those intrinsic to the design of the
/// chain and those to be interpreted by the active chain engine.
pub struct Scheme {
    /// User friendly scheme name
    pub name: String,
    /// What engine are we using for this?
    pub engine: Arc<dyn CodeChainEngine>,
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

    /// Genesis state as plain old data.
    genesis_accounts: PodAccounts,
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
    // create an instance of an CodeChain state machine, minus consensus logic.
    fn machine(_engine_scheme: &cjson::scheme::Engine, params: CommonParams) -> CodeChainMachine {
        CodeChainMachine::new(params)
    }

    /// Convert engine scheme into a arc'd Engine of the right underlying type.
    /// TODO avoid this hard-coded nastiness - use dynamic-linked plugin framework instead.
    fn engine(engine_scheme: cjson::scheme::Engine, params: CommonParams) -> Arc<dyn CodeChainEngine> {
        let machine = Self::machine(&engine_scheme, params);

        match engine_scheme {
            cjson::scheme::Engine::Null => Arc::new(NullEngine::new(machine)),
            cjson::scheme::Engine::Solo(solo) => Arc::new(Solo::new(solo.params.into(), machine)),
            cjson::scheme::Engine::Tendermint(tendermint) => Tendermint::new(tendermint.params.into(), machine),
        }
    }

    fn initialize_state(&self, db: StateDB, genesis_params: CommonParams) -> Result<StateDB, Error> {
        let root = BLAKE_NULL_RLP;
        let (db, root) = self.initialize_accounts(db, root)?;
        let (db, root) = self.initialize_modules(db, root, genesis_params)?;
        let (db, root) = self.engine.initialize_genesis_state(db, root)?;

        *self.state_root_memo.write() = root;
        Ok(db)
    }

    fn initialize_accounts<DB: AsHashDB>(&self, mut db: DB, mut root: H256) -> StateResult<(DB, H256)> {
        // basic accounts in scheme.
        {
            let mut t = TrieFactory::create(db.as_hashdb_mut(), &mut root);

            for (address, account) in &*self.genesis_accounts {
                let r = t.insert(&**address, &account.rlp_bytes());
                debug_assert_eq!(Ok(None), r);
                r?;
            }
        }

        Ok((db, root))
    }

    fn initialize_modules<DB: AsHashDB>(
        &self,
        mut db: DB,
        mut root: H256,
        genesis_params: CommonParams,
    ) -> Result<(DB, H256), Error> {
        // Here we need module initialization
        let global_metadata = Metadata::new(genesis_params);
        {
            let mut t = TrieFactory::from_existing(db.as_hashdb_mut(), &mut root)?;
            let address = MetadataAddress::new();

            let r = t.insert(&*address, &global_metadata.rlp_bytes());
            debug_assert_eq!(Ok(None), r);
            r?;
        }

        Ok((db, root))
    }

    pub fn check_genesis_root(&self, db: &dyn HashDB) -> bool {
        if db.is_empty() {
            return true
        }
        db.contains(&self.state_root())
    }

    /// Ensure that the given state DB has the trie nodes in for the genesis state.
    pub fn ensure_genesis_state(&self, db: StateDB) -> Result<StateDB, Error> {
        if !self.check_genesis_root(db.as_hashdb()) {
            return Err(SchemeError::InvalidState.into())
        }

        if db.as_hashdb().contains(&self.state_root()) {
            return Ok(db)
        }

        Ok(self.initialize_state(db, self.genesis_params())?)
    }

    pub fn check_genesis_common_params<HP: HeaderProvider>(&self, chain: &HP) -> Result<(), Error> {
        let genesis_header = self.genesis_header();
        let genesis_header_hash = genesis_header.hash();
        let header =
            chain.block_header(&genesis_header_hash).ok_or_else(|| Error::Scheme(SchemeError::InvalidCommonParams))?;
        let extra_data = header.extra_data();
        let common_params_hash = blake256(&self.genesis_params().rlp_bytes()).to_vec();
        if extra_data != &common_params_hash {
            return Err(Error::Scheme(SchemeError::InvalidCommonParams))
        }
        Ok(())
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

    /// Get common blockchain parameters.
    pub fn genesis_params(&self) -> CommonParams {
        *self.engine.machine().genesis_common_params()
    }

    /// Get the header of the genesis block.
    pub fn genesis_header(&self) -> Header {
        let mut header: Header = Default::default();
        header.set_parent_hash(self.parent_hash);
        header.set_timestamp(self.timestamp);
        header.set_number(0);
        header.set_author(self.author);
        header.set_transactions_root(self.transactions_root);
        header.set_extra_data(blake256(&self.genesis_params().rlp_bytes()).to_vec());
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

    pub fn genesis_accounts(&self) -> Vec<Address> {
        self.genesis_accounts.keys().cloned().collect()
    }
}

/// Load from JSON object.
fn load_from(s: cjson::scheme::Scheme) -> Result<Scheme, Error> {
    let g = Genesis::from(s.genesis);
    let GenericSeal(seal_rlp) = g.seal.into();
    let params = CommonParams::from(s.params);
    params.verify().map_err(|reason| Error::Syntax(SyntaxError::InvalidCustomAction(reason)))?;
    let engine = Scheme::engine(s.engine, params);

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
        genesis_accounts: s.accounts.into(),
    };

    // use memoized state root if provided.
    match g.state_root {
        Some(root) => *s.state_root_memo.get_mut() = root,
        None => {
            let db = StateDB::new_with_memorydb();
            let _ = s.initialize_state(db, s.genesis_params())?;
        }
    }

    Ok(s)
}

#[cfg(test)]
mod tests {
    use ccrypto::Blake;

    use super::*;

    #[test]
    fn extra_data_of_genesis_header_is_hash_of_common_params() {
        let scheme = Scheme::new_test();
        let common_params = scheme.genesis_params();
        let hash_of_common_params = H256::blake(&common_params.rlp_bytes()).to_vec();

        let genesis_header = scheme.genesis_header();
        let result = genesis_header.extra_data();
        assert_eq!(&hash_of_common_params, result);
    }
}
