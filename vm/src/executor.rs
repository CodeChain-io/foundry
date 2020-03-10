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
#![allow(dead_code)]

use ckey::{verify, Ed25519Public as Public, Signature};
use ctypes::transaction::HashingError;
use ctypes::{BlockNumber, Tracker};
use primitives::H256;

const DEFAULT_MAX_MEMORY: usize = 1024;

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum TimelockType {
    Block = 0x01,
    BlockAge = 0x02,
    Time = 0x03,
    TimeAge = 0x04,
}

pub struct Config {
    pub max_memory: usize,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            max_memory: DEFAULT_MAX_MEMORY,
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum ScriptResult {
    Fail,
    Unlocked,
    Burnt,
}

#[derive(Debug, PartialEq)]
pub enum RuntimeError {
    OutOfMemory,
    IndexOutOfBound,
    StackUnderflow,
    TypeMismatch,
    InvalidFilter,
    InvalidSigCount,
    InvalidTimelockType,
    InvalidSig,
    InvalidPubkey,
}

impl From<HashingError> for RuntimeError {
    fn from(error: HashingError) -> Self {
        match error {
            HashingError::InvalidFilter => RuntimeError::InvalidFilter,
        }
    }
}

#[derive(Clone)]
struct Item(Vec<u8>);

impl Item {
    fn len(&self) -> usize {
        self.0.len()
    }

    fn assert_len(self, len: usize) -> Result<Self, RuntimeError> {
        if self.len() == len {
            Ok(self)
        } else {
            Err(RuntimeError::TypeMismatch)
        }
    }
}

impl AsRef<[u8]> for Item {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl From<bool> for Item {
    fn from(val: bool) -> Item {
        if val {
            Item(vec![1])
        } else {
            Item(vec![])
        }
    }
}

impl From<Item> for bool {
    fn from(item: Item) -> Self {
        item.as_ref().iter().any(|b| b != &0)
    }
}

struct Stack {
    stack: Vec<Item>,
    memory_usage: usize,
    config: Config,
}

impl Stack {
    fn new(config: Config) -> Self {
        Self {
            stack: Vec::new(),
            memory_usage: 0,
            config,
        }
    }

    /// Returns true if value is successfully pushed
    fn push(&mut self, val: Item) -> Result<(), RuntimeError> {
        if self.memory_usage + val.len() > self.config.max_memory {
            Err(RuntimeError::OutOfMemory)
        } else {
            self.memory_usage += val.len();
            self.stack.push(val);
            Ok(())
        }
    }

    fn pop(&mut self) -> Result<Item, RuntimeError> {
        let item = self.stack.pop();
        self.memory_usage -= item.as_ref().map_or(0, Item::len);
        item.ok_or(RuntimeError::StackUnderflow)
    }

    fn len(&self) -> usize {
        self.stack.len()
    }

    fn get(&self, index: usize) -> Result<Item, RuntimeError> {
        self.stack.get(index).cloned().ok_or(RuntimeError::IndexOutOfBound)
    }

    fn remove(&mut self, index: usize) -> Result<Item, RuntimeError> {
        if index < self.stack.len() {
            let item = self.stack.remove(index);
            self.memory_usage -= item.len();
            Ok(item)
        } else {
            Err(RuntimeError::IndexOutOfBound)
        }
    }
}

fn read_u64(value_item: Item) -> Result<u64, RuntimeError> {
    if value_item.len() > 8 {
        return Err(RuntimeError::TypeMismatch)
    }
    let mut value_bytes = [0u8; 8];
    value_bytes[(8 - value_item.len())..8].copy_from_slice(value_item.as_ref());
    Ok(u64::from_be_bytes(value_bytes))
}

#[inline]
fn check_multi_sig(tx_hash: &H256, mut pubkey: Vec<Public>, mut signatures: Vec<Signature>) -> bool {
    while let Some(sig) = signatures.pop() {
        loop {
            let public = match pubkey.pop() {
                None => return false,
                Some(public) => public,
            };
            if verify(&sig, &tx_hash, &public) {
                break
            }
        }
    }
    true
}

pub trait ChainTimeInfo {
    /// Get the block height of the transaction.
    fn transaction_block_age(&self, tracker: &Tracker, parent_block_number: BlockNumber) -> Option<u64>;

    /// Get the how many seconds elapsed since transaction is confirmed, according to block timestamp.
    fn transaction_time_age(&self, tracker: &Tracker, parent_timestamp: u64) -> Option<u64>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn convert_true() {
        let item: Item = true.into();
        assert_eq!(vec![1], item.as_ref());
        let result: bool = item.into();
        assert!(result);
    }

    #[test]
    fn convert_false() {
        let item: Item = false.into();
        assert_eq!(Vec::<u8>::new(), item.as_ref());
        let result: bool = item.into();
        assert!(!result);
    }

    #[test]
    fn false_if_all_bit_is_zero() {
        let item = Item(vec![0, 0, 0, 0, 0, 0, 0]);
        let result: bool = item.into();
        assert!(!result);
    }

    #[test]
    fn true_if_at_least_one_bit_is_not_zero() {
        let item = Item(vec![0, 0, 0, 1, 0, 0, 0]);
        let result: bool = item.into();
        assert!(result);
    }

    #[test]
    fn read_0_0_0_0_0_0_0_1() {
        assert_eq!(Ok(0x0000_0000_0000_0001), read_u64(Item(vec![0, 0, 0, 0, 0, 0, 0, 1])));
    }

    #[test]
    fn read_1() {
        assert_eq!(Ok(0x01), read_u64(Item(vec![1])));
    }

    #[test]
    fn read_f() {
        assert_eq!(Ok(0x0f), read_u64(Item(vec![0xf])))
    }

    #[test]
    fn read_1_0() {
        assert_eq!(Ok(0x0100), read_u64(Item(vec![1, 0])));
    }

    #[test]
    fn read_1_0_0() {
        assert_eq!(Ok(0x0001_0000), read_u64(Item(vec![1, 0, 0])));
    }

    #[test]
    fn read_1_0_0_0() {
        assert_eq!(Ok(0x0100_0000), read_u64(Item(vec![1, 0, 0, 0])));
    }

    #[test]
    fn read_1_0_0_0_0_0_0_0() {
        assert_eq!(Ok(0x0100_0000_0000_0000), read_u64(Item(vec![1, 0, 0, 0, 0, 0, 0, 0])));
    }

    #[test]
    fn read_1_0_0_0_0_0_1_0() {
        assert_eq!(Ok(0x0100_0000_0000_0100), read_u64(Item(vec![1, 0, 0, 0, 0, 0, 1, 0])));
    }
}

#[cfg(test)]
mod tests_check_multi_sig {
    use ckey::{sign, Ed25519KeyPair as KeyPair, Generator, KeyPairTrait, Random};

    use super::*;

    #[test]
    fn valid_2_of_3_110() {
        let key_pair1: KeyPair = Random.generate().unwrap();
        let key_pair2: KeyPair = Random.generate().unwrap();
        let key_pair3: KeyPair = Random.generate().unwrap();
        let pubkey1 = *key_pair1.public();
        let pubkey2 = *key_pair2.public();
        let pubkey3 = *key_pair3.public();
        let message = H256::random();
        let signature1 = sign(&message, key_pair1.private());
        let signature2 = sign(&message, key_pair2.private());

        assert!(check_multi_sig(&message, vec![pubkey1, pubkey2, pubkey3], vec![signature1, signature2]));
    }

    #[test]
    fn valid_2_of_3_101() {
        let key_pair1: KeyPair = Random.generate().unwrap();
        let key_pair2: KeyPair = Random.generate().unwrap();
        let key_pair3: KeyPair = Random.generate().unwrap();
        let pubkey1 = *key_pair1.public();
        let pubkey2 = *key_pair2.public();
        let pubkey3 = *key_pair3.public();
        let message = H256::random();
        let signature1 = sign(&message, key_pair1.private());
        let signature3 = sign(&message, key_pair3.private());

        assert!(check_multi_sig(&message, vec![pubkey1, pubkey2, pubkey3], vec![signature1, signature3]));
    }

    #[test]
    fn valid_2_of_3_011() {
        let key_pair1: KeyPair = Random.generate().unwrap();
        let key_pair2: KeyPair = Random.generate().unwrap();
        let key_pair3: KeyPair = Random.generate().unwrap();
        let pubkey1 = *key_pair1.public();
        let pubkey2 = *key_pair2.public();
        let pubkey3 = *key_pair3.public();
        let message = H256::random();
        let signature2 = sign(&message, key_pair2.private());
        let signature3 = sign(&message, key_pair3.private());

        assert!(check_multi_sig(&message, vec![pubkey1, pubkey2, pubkey3], vec![signature2, signature3]));
    }

    #[test]
    fn invalid_2_of_2_if_order_is_different() {
        let key_pair1: KeyPair = Random.generate().unwrap();
        let key_pair2: KeyPair = Random.generate().unwrap();
        let pubkey1 = *key_pair1.public();
        let pubkey2 = *key_pair2.public();
        let message = H256::random();
        let signature1 = sign(&message, key_pair1.private());
        let signature2 = sign(&message, key_pair2.private());

        assert!(!check_multi_sig(&message, vec![pubkey2, pubkey1], vec![signature1, signature2]));
    }
}
