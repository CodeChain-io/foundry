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

extern crate codechain_crypto as ccrypto;
extern crate codechain_key as ckey;
extern crate codechain_types as ctypes;
extern crate codechain_vm as cvm;
extern crate primitives;

mod common;

use common::TestClient;
use ctypes::transaction::{AssetOutPoint, AssetTransferInput, ShardTransaction};
use cvm::{execute, RuntimeError, ScriptResult, VMConfig};
use cvm::{Instruction, TimelockType};
use primitives::H160;

#[cfg(test)]
fn dummy_tx() -> ShardTransaction {
    ShardTransaction::ShardStore {
        network_id: Default::default(),
        shard_id: 0,
        content: "".to_string(),
    }
}

fn dummy_input() -> AssetTransferInput {
    AssetTransferInput {
        prev_out: AssetOutPoint {
            tracker: Default::default(),
            index: 0,
            asset_type: H160::default(),
            shard_id: 0,
            quantity: 0,
        },
        timelock: None,
        lock_script: Vec::new(),
        unlock_script: Vec::new(),
    }
}

#[test]
fn timelock_invalid_value() {
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![0, 0, 0, 0, 0, 0, 0, 0, 0]), Instruction::ChkTimelock(TimelockType::Block)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &TestClient::default(),
            0,
            0
        ),
        Err(RuntimeError::TypeMismatch)
    )
}

#[test]
fn timelock_block_number_success() {
    let client = TestClient::new(None, None);
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![10]), Instruction::ChkTimelock(TimelockType::Block)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            10,
            0
        ),
        Ok(ScriptResult::Unlocked)
    )
}

#[test]
fn timelock_block_number_fail() {
    let client = TestClient::new(None, None);
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![10]), Instruction::ChkTimelock(TimelockType::Block)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            9,
            0
        ),
        Ok(ScriptResult::Fail)
    )
}

#[test]
fn timelock_block_timestamp_success() {
    // 0x5BD02BF2, 2018-10-24T08:23:14+00:00
    let client = TestClient::new(None, None);
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![0x00, 0x5B, 0xD0, 0x2B, 0xF2]), Instruction::ChkTimelock(TimelockType::Time)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            0,
            1_540_369_394
        ),
        Ok(ScriptResult::Unlocked)
    )
}

#[test]
fn timelock_block_timestamp_fail() {
    // 0x5BD02BF1, 2018-10-24T08:23:13+00:00
    let client = TestClient::new(None, None);
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![0x00, 0x5B, 0xD0, 0x2B, 0xF2]), Instruction::ChkTimelock(TimelockType::Time)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            0,
            1_540_369_393
        ),
        Ok(ScriptResult::Fail)
    )
}

#[test]
fn timelock_block_age_fail_due_to_none() {
    let client = TestClient::new(None, None);
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![1]), Instruction::ChkTimelock(TimelockType::BlockAge)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            0,
            0
        ),
        Ok(ScriptResult::Fail)
    )
}

#[test]
fn timelock_block_age_fail() {
    let client = TestClient::new(Some(4), None);
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![5]), Instruction::ChkTimelock(TimelockType::BlockAge)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            0,
            0
        ),
        Ok(ScriptResult::Fail)
    )
}

#[test]
fn timelock_block_age_success() {
    let client = TestClient::new(Some(5), None);
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![5]), Instruction::ChkTimelock(TimelockType::BlockAge)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            0,
            0
        ),
        Ok(ScriptResult::Unlocked)
    )
}

#[test]
fn timelock_time_age_fail_due_to_none() {
    let client = TestClient::new(None, None);
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![0x27, 0x8D, 0x00]), Instruction::ChkTimelock(TimelockType::TimeAge)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            0,
            0
        ),
        Ok(ScriptResult::Fail)
    )
}

#[test]
fn timelock_time_age_fail() {
    // 0x278D00 seconds = 2592000 seconds = 30 days
    let client = TestClient::new(None, Some(2_591_999));
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![0x27, 0x8D, 0x00]), Instruction::ChkTimelock(TimelockType::TimeAge)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            0,
            0
        ),
        Ok(ScriptResult::Fail)
    )
}

#[test]
fn timelock_time_age_success() {
    let client = TestClient::new(None, Some(2_592_000));
    assert_eq!(
        execute(
            &[],
            &[],
            &[Instruction::PushB(vec![0x27, 0x8D, 0x00]), Instruction::ChkTimelock(TimelockType::TimeAge)],
            &dummy_tx(),
            VMConfig::default(),
            &dummy_input(),
            false,
            &client,
            0,
            0
        ),
        Ok(ScriptResult::Unlocked)
    )
}
