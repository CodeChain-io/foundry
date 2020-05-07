use crate::types::BlockId;
use ckey::Ed25519Public as Public;
use ctypes::Header;
use std::collections::{BTreeSet, HashMap};

// From module Account
pub fn add_balance(_address: &Public, _val: u64) {
    unimplemented!();
}

// From module Staking
pub fn get_stakes() -> HashMap<Public, u64> {
    unimplemented!();
}

// From module Staking
pub fn current_term_id(_block_number: u64) -> u64 {
    unimplemented!();
}

// From module Staking
pub fn is_term_changed(_current_block_number: u64) -> bool {
    unimplemented!();
}

// From module Staking
pub fn last_term_finished_block_num(_block_number: u64) -> u64 {
    unimplemented!();
}

// From module Staking
pub fn get_current_validators(_block_id: BlockId) -> Vec<Public> {
    unimplemented!();
}

// From module Staking
pub fn get_previous_validators(_block_id: BlockId) -> Vec<Public> {
    unimplemented!();
}

pub struct Banned(BTreeSet<Public>);

impl Banned {
    pub fn is_banned(&self, public: &Public) -> bool {
        self.0.contains(public)
    }
}

pub fn get_banned_validators() -> Banned {
    unimplemented!();
}

pub fn get_chain_history_access() -> Box<dyn ChainHistoryAccess> {
    unimplemented!()
}

pub trait ChainHistoryAccess {
    fn get_block_header(&self, block_id: BlockId) -> Option<Header>;
}
