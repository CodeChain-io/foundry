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

pub const NETWORK_ID: &str = "tc";

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

macro_rules! set_module_level_state {
    // base cases
    ($state:expr, [(key: $key:expr => datum: $datum:expr)]) => {
        assert_eq!(Ok(()), $state.set_datum(&$key, $datum));
    };
    ($state:expr, [(key: $key:expr => datum_str: $datum_str:expr)]) => {
        set_module_level_state!($state, [(key: $key => datum: String::from($datum_str).into_bytes())]);
    };
    ($state:expr, [(key: $key:expr => None)]) => {
        assert_eq!(Ok(()), $state.remove(&$key));
    };
    // recursion
    ($state:expr, [$head:tt, $($tail:tt),+ $(,)?]) => {
        set_module_level_state!($state, [$head]);
        set_module_level_state!($state, [$($tail),+]);
    };
}

macro_rules! check_module_level_state {
    // base cases
    ($state:expr, [(key: $key:expr => datum: $datum:expr)]) => {
        assert_eq!(Ok(true), $state.has_key(&$key));
        assert_eq!(Some($datum), $state.get_datum(&$key).unwrap().map(|datum| datum.content()));
    };
    ($state:expr, [(key: $key:expr => datum_str: $datum_str:expr)]) => {
        check_module_level_state!($state, [(key: $key => datum: String::from($datum_str).into_bytes())]);
    };
    ($state:expr, [(key: $key:expr => None)]) => {
        assert_eq!(Ok(false), $state.has_key(&$key));
        assert_eq!(None, $state.get_datum(&$key).unwrap());
    };
    ($state:expr, [(key: $key:expr => Some)]) => {
        assert_eq!(Ok(true), $state.has_key(&$key));
    };
    // recursion
    ($state:expr, [$head:tt, $($tail:tt),+ $(,)?]) => {
        check_module_level_state!($state, [$head]);
        check_module_level_state!($state, [$($tail),+]);
    };
}

macro_rules! module_level {
    ($state:expr, {check: $content:tt}) => {
        check_module_level_state!($state, $content);
    };
    ($state:expr, {set: $content:tt}) => {
        set_module_level_state!($state, $content);
    };
    //recursion
    ($state:expr, {$opl:ident: $contentl:tt, $($opr:ident: $contentr:tt),+ $(,)?}) => {
        module_level!($state, {$opl: $contentl});
        module_level!($state, {$($opr: $contentr),+});
    }
}

macro_rules! set_top_level_state {
    // base cases
    ($state:expr, [(account: $addr:expr => balance: $quantity:expr)]) => {
        assert_eq!(Ok(()), $state.set_balance(&$addr, $quantity));
    };
    ($state:expr, [(account: $addr:expr => seq: $seq:expr)]) => {
        assert_eq!(Ok(()), $state.set_seq(&$addr, $seq));
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
    //recursion
    ($state:expr, [$head:tt, $($tail:tt),+]) => {
        check_top_level_state!($state, [$head]);
        check_top_level_state!($state, [$($tail),+]);
    }
}
