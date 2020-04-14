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
    ($state:expr, [(account: $addr:expr => seq++)]) => {
        assert_eq!(Ok(()), $state.inc_seq(&$addr));
    };
    ($state:expr, [(account: $addr:expr => Kill)]) => {
        $state.kill_account(&$addr);
    };
    ($state:expr, [(account: $addr:expr => balance: add $quantity:expr)]) => {
        assert_eq!(Ok(()), $state.add_balance(&$addr, $quantity));
    };
    ($state:expr, [(account: $addr:expr => balance: sub $quantity:expr)]) => {
        assert_eq!(Ok(()), $state.sub_balance(&$addr, $quantity));
    };
    ($state:expr, [(account: $from:expr => transfer: $to:expr, $quantity:expr)]) => {
        assert_eq!(Ok(()), $state.transfer_balance(&$from, &$to, $quantity));
    };
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*])]) => {
        set_top_level_state!($state, [(shard: $shard_id => owners: vec![$($owner),*])]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: $owners:expr)]) => {
        assert_eq!(Ok(()), $state.create_shard_level_state($shard_id, $owners));
    };
    ($state:expr, [(metadata: shards: $number_of_shards:expr)]) => {
        assert_eq!(Ok(()), $state.set_number_of_shards($number_of_shards));
    };
    // recursion
    ($state:expr, [$head:tt, $($tail:tt),+ $(,)?]) => {
        set_top_level_state!($state, [$head]);
        set_top_level_state!($state, [$($tail),+]);
    };
}

macro_rules! check_top_level_state {
    // base cases
    ($state:expr, [(account: $addr:expr => seq: $seq:expr)]) => {
        assert_eq!(Ok($seq), $state.seq(&$addr));
    };
    ($state:expr, [(account: $addr:expr => balance: $balance:expr)]) => {
        assert_eq!(Ok($balance), $state.balance(&$addr));
    };
    ($state:expr, [(account: $addr:expr)]) => {
        check_top_level_state!($state, [(account: $addr => seq: 0, balance: 0)]);
    };
    ($state:expr, [(account: $addr:expr => None)]) => {
        assert_eq!(Ok(false), $state.account_exists(&$addr));
        assert_eq!(Ok(false), $state.account_exists_and_not_null(&$addr));
    };
    ($state:expr, [(account: $addr:expr => Some)]) => {
        assert_eq!(Ok(true), $state.account_exists(&$addr));
        assert_eq!(Ok(true), $state.account_exists_and_not_null(&$addr));
    };
    ($state:expr, [(account: $addr:expr => $head:ident: $head_val:expr, $($tail:ident: $tail_val:expr),+)]) => {
        check_top_level_state!($state, [(account: $addr => $head: $head_val)]);
        check_top_level_state!($state, [(account: $addr => $($tail: $tail_val),+)]);
    };
    //recursion
    ($state:expr, [$head:tt, $($tail:tt),+ $(,)?]) => {
        check_top_level_state!($state, [$head]);
        check_top_level_state!($state, [$($tail),+]);
    }
}

macro_rules! check_shard_level_state {
    // base cases
    ($state:expr, [(text: ($tx_hash:expr) => { content: $content: expr})]) => {
        let stored_text = $state.text($tx_hash)
            .expect(&format!("Cannot read Text from {}:{}", $state.shard_id(), $tx_hash))
            .expect(&format!("Text for {}:{} not exist", $state.shard_id(), $tx_hash));
        assert_eq!($content, stored_text.content());
    };
    // recursion
    ($state:expr, [$head:tt, $($tail:tt),+]) => {
        check_shard_level_state!($state, [$head]);
        check_shard_level_state!($state, [$($tail),+]);
    }
}

macro_rules! top_level {
    ($state:expr, {check: $content:tt}) => {
        check_top_level_state!($state, $content);
    };
    ($state:expr, {set: $content:tt}) => {
        set_top_level_state!($state, $content);
    };
    //recursion
    ($state:expr, {$opl:ident: $contentl:tt, $($opr:ident: $contentr:tt),+ $(,)?}) => {
        top_level!($state, {$opl: $contentl});
        top_level!($state, {$($opr: $contentr),+});
    }
}
