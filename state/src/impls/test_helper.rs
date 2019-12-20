// Copyright 2018-2019 Kodebox, Inc.
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

macro_rules! set_regular_key {
    ($key:expr) => {
        $crate::ctypes::transaction::Action::SetRegularKey {
            key: $key,
        }
    };
}

macro_rules! store {
    ($content:expr, $certifier:expr, $signature:expr) => {
        $crate::ctypes::transaction::Action::Store {
            content: $content,
            certifier: $certifier,
            signature: $signature,
        }
    };
}

macro_rules! remove {
    ($hash:expr, $signature:expr) => {
        $crate::ctypes::transaction::Action::Remove {
            hash: $hash,
            signature: $signature,
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
    ($state: expr, []) => {
    };
    ($state:expr, [(regular_key: $signer:expr => $key:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(()), $state.set_regular_key(&$signer, &$key));

        set_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(account: $addr:expr => balance: $quantity:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(()), $state.set_balance(&$addr, $quantity));

        set_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(account: $addr:expr => seq: $seq:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(()), $state.set_seq(&$addr, $seq));

        set_top_level_state!($state, [$($x),*]);
    };
}

macro_rules! check_top_level_state {
    ($state: expr, []) => { };
    ($state:expr, [(account: $addr:expr => (seq: $seq:expr, balance: $balance:expr)) $(,$x:tt)*]) => {
        assert_eq!(Ok($seq), $state.seq(&$addr));
        assert_eq!(Ok($balance), $state.balance(&$addr));

        check_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(account: $addr:expr => (seq: $seq:expr, balance: $balance:expr, key: $key:expr)) $(,$x:tt)*]) => {
        assert_eq!(Ok(Some($key)), $state.regular_key(&$addr));
        check_top_level_state!($state, [(account: $addr => (seq: $seq, balance: $balance)) $(,$x)*]);
    };
    ($state:expr, [(account: $addr:expr => (seq: $seq:expr, balance: $balance:expr, key)) $(,$x:tt)*]) => {
        assert_eq!(Ok(None), $state.regular_key(&$addr));
        check_top_level_state!($state, [(account: $addr => (seq: $seq, balance: $balance)) $(,$x)*]);
    };
    ($state:expr, [(account: $addr:expr) $(,$x:tt)*]) => {
        check_top_level_state!($state, [(account: $addr => (seq: 0, balance: 0)) $(,$x)*]);
    };
    ($state:expr, [(text: $tx_hash:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(None), $state.text($tx_hash));

        check_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(text: $tx_hash:expr => { content: $content:expr, certifier: $certifier:expr }) $(,$x:tt)*]) => {
        let text = $crate::Text::new($content, $certifier);
        assert_eq!(Ok(Some(text)), $state.text($tx_hash));

        check_top_level_state!($state, [$($x),*]);
    };
}
