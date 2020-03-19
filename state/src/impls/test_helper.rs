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

#![macro_use]

use ctypes::ShardId;

pub const NETWORK_ID: &str = "tc";
pub const SHARD_ID: ShardId = 0;

macro_rules! pay {
    ($receiver:expr, $quantity:expr) => {
        $crate::ctypes::transaction::Action::Pay {
            receiver: $receiver,
            quantity: $quantity,
        }
    };
}

macro_rules! set_shard_owners {
    (shard_id: $shard_id:expr, $owners:expr) => {
        $crate::ctypes::transaction::Action::SetShardOwners {
            shard_id: $shard_id,
            owners: $owners,
        }
    };
    ($owners:expr) => {
        $crate::ctypes::transaction::Action::SetShardOwners {
            shard_id: $crate::impls::test_helper::SHARD_ID,
            owners: $owners,
        }
    };
}

macro_rules! set_shard_users {
    ($users:expr) => {
        $crate::ctypes::transaction::Action::SetShardUsers {
            shard_id: $crate::impls::test_helper::SHARD_ID,
            users: $users,
        }
    };
}

macro_rules! transaction {
    (fee: $fee:expr, $action:expr) => {
        transaction!(seq: 0, fee: $fee, $action)
    };
    (seq: $seq:expr, fee: $fee:expr, $action:expr) => {
        $crate::ctypes::transaction::Transaction {
            seq: $seq,
            fee: $fee,
            network_id: $crate::impls::test_helper::NETWORK_ID.into(),
            action: $action,
        }
    };
}

macro_rules! set_top_level_state {
    // base cases
    ($state:expr, [(account: $addr:expr => balance: $quantity:expr)]) => {
        assert_eq!(Ok(()), $state.set_balance(&$addr, $quantity));
    };
    ($state:expr, [(account: $addr:expr => seq: $seq:expr)]) => {
        assert_eq!(Ok(()), $state.set_seq(&$addr, $seq));
    };
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*])]) => {
        set_top_level_state!($state, [(shard: $shard_id => owners: [$($owner),*], users: Vec::new())]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*], users: [$($user:expr),*])]) => {
        set_top_level_state!($state, [(shard: $shard_id => owners: [$($owner),*], users: vec![$($user),*])]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*], users: $users:expr)]) => {
        set_top_level_state!($state, [(shard: $shard_id => owners: vec![$($owner),*], users: $users)]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: $owners:expr, users: $users:expr)]) => {
        assert_eq!(Ok(()), $state.create_shard_level_state($shard_id, $owners, $users));
    };
    ($state:expr, [(metadata: shards: $number_of_shards:expr)]) => {
        assert_eq!(Ok(()), $state.set_number_of_shards($number_of_shards));
    };
    // recursion
    ($state:expr, [$head:tt, $($tail:tt),+]) => {
        set_top_level_state!($state, [$head]);
        set_top_level_state!($state, [$($tail),+]);
    };
}

macro_rules! check_top_level_state {
    // base cases
    ($state:expr, [(account: $addr:expr => (seq: $seq:expr, balance: $balance:expr))]) => {
        assert_eq!(Ok($seq), $state.seq(&$addr));
        assert_eq!(Ok($balance), $state.balance(&$addr));
    };
    ($state:expr, [(account: $addr:expr)]) => {
        check_top_level_state!($state, [(account: $addr => (seq: 0, balance: 0))]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*])]) => {
        check_top_level_state!($state, [(shard: $shard_id => owners: vec![$($owner,)*])]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: $owners:expr)]) => {
        assert_eq!(Ok(Some($owners)), $state.shard_owners($shard_id));
    };
    ($state:expr, [(shard: $shard_id:expr => owners: $owners:expr, users: $users:expr)]) => {
        assert_eq!(Ok(Some($users)), $state.shard_users($shard_id));
    };
    ($state:expr, [(shard: $shard_id:expr)]) => {
        assert_eq!(Ok(None), $state.shard_root($shard_id));
    };
    ($state:expr, [(shard_text: ($shard_id:expr, $tracker:expr))]) => {
        assert_eq!(Ok(None), $state.shard_text($shard_id, $tracker));
    };
    //recursion
    ($state:expr, [$head:tt, $($tail:tt),+]) => {
        check_top_level_state!($state, [$head]);
        check_top_level_state!($state, [$($tail),+]);
    }
}

macro_rules! check_shard_level_state {
    // base cases
    ($state:expr, [(text: ($tracker:expr) => { content: $content: expr})]) => {
        let stored_text = $state.text($tracker)
            .expect(&format!("Cannot read Text from {}:{}", $state.shard_id(), $tracker))
            .expect(&format!("Text for {}:{} not exist", $state.shard_id(), $tracker));
        assert_eq!($content, stored_text.content());
    };
    // recursion
    ($state:expr, [$head:tt, $($tail:tt),+]) => {
        check_shard_level_state!($state, [$head]);
        check_shard_level_state!($state, [$($tail),+]);
    }
}
