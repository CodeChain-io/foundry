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

macro_rules! set_regular_key {
    ($key:expr) => {
        $crate::ctypes::transaction::Action::SetRegularKey {
            key: $key,
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
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*]) $(,$x:tt)*]) => {
        set_top_level_state!($state, [(shard: $shard_id => owners: [$($owner),*], users: Vec::new()) $(,$x)*]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*], users: [$($user:expr),*]) $(,$x:tt)*]) => {
        set_top_level_state!($state, [(shard: $shard_id => owners: [$($owner),*], users: vec![$($user),*]) $(,$x)*]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*], users: $users:expr) $(,$x:tt)*]) => {
        set_top_level_state!($state, [(shard: $shard_id => owners: vec![$($owner),*], users: $users) $(,$x)*]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: $owners:expr, users: $users:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(()), $state.create_shard_level_state($shard_id, $owners, $users));

        set_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(metadata: shards: $number_of_shards:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(()), $state.set_number_of_shards($number_of_shards));

        set_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($shard_id:expr, $asset_type:expr) => { supply: $supply:expr, metadata: $metadata:expr }) $(,$x:tt)*]) => {
        assert_eq!(Ok((true)), $state.create_asset_scheme($shard_id, $asset_type, $metadata, $supply, None, None, Vec::new(), Vec::new()));

        set_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($shard_id:expr, $asset_type:expr) => { supply: $supply:expr, metadata: $metadata:expr, approver: $approver:expr }) $(,$x:tt)*]) => {
        assert_eq!(Ok((true)), $state.create_asset_scheme($shard_id, $asset_type, $metadata, $supply, $approver, None, Vec::new(), Vec::new()));

        set_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(asset: ($shard:expr, $tx_hash:expr, $index:expr) => { asset_type: $asset_type: expr, quantity: $quantity:expr, lock_script_hash: $lock_script_hash:expr }) $(,$x:tt)*]) => {
        assert_eq!(Ok((true)), $state.create_asset($shard, $tx_hash, $index, $asset_type, $lock_script_hash, Vec::new(), $quantity));

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
    ($state:expr, [(shard: $shard_id:expr => owners: [$($owner:expr),*]) $(,$x:tt)*]) => {
        check_top_level_state!($state, [(shard: $shard_id => owners: vec![$($owner,)*]) $(,$x)*]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: $owners:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(Some($owners)), $state.shard_owners($shard_id));

        check_top_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(shard: $shard_id:expr => owners: $owners:expr, users: $users:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(Some($users)), $state.shard_users($shard_id));

        check_top_level_state!($state, [(shard: $shard_id => owners: $owners) $(,$x)*]);
    };
    ($state:expr, [(shard: $shard_id:expr) $(,$x:tt)*]) => {
        assert_eq!(Ok(None), $state.shard_root($shard_id));

        check_top_level_state!($state, [$($x),*]);
    };
}

macro_rules! check_shard_level_state {
    ($state: expr, []) => { };
    ($state:expr, [(scheme: ($asset_type:expr) => { supply: $supply:expr }) $(,$x:tt)*]) => {
        let scheme = $state.asset_scheme($asset_type)
            .expect(&format!("Cannot read AssetScheme from {}:{}", $state.shard_id(), $asset_type))
            .expect(&format!("AssetScheme for {}:{} not exist", $state.shard_id(), $asset_type));
        assert_eq!($supply, scheme.supply());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($asset_type:expr) => { metadata: $metadata:expr, supply: $supply:expr }) $(,$x:tt)*]) => {
        let scheme = $state.asset_scheme($asset_type)
            .expect(&format!("Cannot read AssetScheme from {}:{}", $state.shard_id(), $asset_type))
            .expect(&format!("AssetScheme for {}:{} not exist", $state.shard_id(), $asset_type));
        assert_eq!(&$metadata, scheme.metadata());
        assert_eq!($supply, scheme.supply());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($asset_type:expr) => { metadata: $metadata:expr, supply: $supply:expr, allowed_script_hashes: $allowed:expr}) $(,$x:tt)*]) => {
        let scheme = $state.asset_scheme($asset_type)
            .expect(&format!("Cannot read AssetScheme from {}:{}", $state.shard_id(), $asset_type))
            .expect(&format!("AssetScheme for {}:{} not exist", $state.shard_id(), $asset_type));
        assert_eq!(&$metadata, scheme.metadata());
        assert_eq!($supply, scheme.supply());
        assert_eq!($allowed, scheme.allowed_script_hashes());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($asset_type:expr) => { metadata: $metadata:expr, supply: $supply:expr, pool: $pool:expr }) $(,$x:tt)*]) => {
        let scheme = $state.asset_scheme($asset_type)
            .expect(&format!("Cannot read AssetScheme from {}:{}", $state.shard_id(), $asset_type))
            .expect(&format!("AssetScheme for {}:{} not exist", $state.shard_id(), $asset_type));
        assert_eq!(&$metadata, scheme.metadata());
        assert_eq!($supply, scheme.supply());
        assert_eq!($pool, scheme.pool());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($asset_type:expr) => { metadata: $metadata:expr, supply: $supply:expr, approver: $approver:expr }) $(,$x:tt)*]) => {
        let scheme = $state.asset_scheme($asset_type)
            .expect(&format!("Cannot read AssetScheme from {}:{}", $state.shard_id(), $asset_type))
            .expect(&format!("AssetScheme for {}:{} not exist", $state.shard_id(), $asset_type));
        assert_eq!(&$metadata, scheme.metadata());
        assert_eq!($supply, scheme.supply());
        assert_eq!(Some(&$approver), scheme.approver().as_ref());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($asset_type:expr) => { metadata: $metadata:expr, supply: $supply:expr, approver: $approver:expr, registrar }) $(,$x:tt)*]) => {
        let scheme = $state.asset_scheme($asset_type)
            .expect(&format!("Cannot read AssetScheme from {}:{}", $state.shard_id(), $asset_type))
            .expect(&format!("AssetScheme for {}:{} not exist", $state.shard_id(), $asset_type));
        assert_eq!(&$metadata, scheme.metadata());
        assert_eq!($supply, scheme.supply());
        assert_eq!(Some(&$approver), scheme.approver().as_ref());
        assert_eq!(&None, scheme.registrar());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($asset_type:expr) => { metadata: $metadata:expr, supply: $supply:expr, approver, registrar: $registrar:expr }) $(,$x:tt)*]) => {
        let scheme = $state.asset_scheme($asset_type)
            .expect(&format!("Cannot read AssetScheme from {}:{}", $state.shard_id(), $asset_type))
            .expect(&format!("AssetScheme for {}:{} not exist", $state.shard_id(), $asset_type));
        assert_eq!(&$metadata, scheme.metadata());
        assert_eq!($supply, scheme.supply());
        assert_eq!(&None, scheme.approver());
        assert_eq!(Some(&$registrar), scheme.registrar().as_ref());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($asset_type:expr) => { metadata: $metadata:expr, supply: $supply:expr, registrar: $registrar:expr }) $(,$x:tt)*]) => {
        let scheme = $state.asset_scheme($asset_type)
            .expect(&format!("Cannot read AssetScheme from {}:{}", $state.shard_id(), $asset_type))
            .expect(&format!("AssetScheme for {}:{} not exist", $state.shard_id(), $asset_type));
        assert_eq!(&$metadata, scheme.metadata());
        assert_eq!($supply, scheme.supply());
        assert_eq!(Some(&$registrar), scheme.registrar().as_ref());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(scheme: ($asset_type:expr)) $(,$x:tt)*]) => {
        assert_eq!(Ok(None), $state.asset_scheme($asset_type));

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(asset: ($tracker:expr, $index:expr) => { asset_type: $asset_type:expr, quantity: $quantity:expr }) $(,$x:tt)*]) => {
        let asset = $state.asset($tracker, $index)
            .expect(&format!("Cannot read Asset from {}:{}:{}", $state.shard_id(), $tracker, $index))
            .expect(&format!("Asset for {}:{}:{} not exist", $state.shard_id(), $tracker, $index));
        assert_eq!(&$asset_type, asset.asset_type());
        assert_eq!($quantity, asset.quantity());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(asset: ($tracker:expr, $index:expr) => { asset_type: $asset_type:expr, quantity: $quantity:expr }) $(,$x:tt)*]) => {
        let asset = $state.asset($tracker, $index)
            .expect(&format!("Cannot read Asset from {}:{}:{}", $state.shard_id(), $tracker, $index))
            .expect(&format!("Asset for {}:{}:{} not exist", $state.shard_id(), $tracker, $index));
        assert_eq!(&$asset_type, asset.asset_type());
        assert_eq!($quantity, asset.quantity());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(asset: ($tracker:expr, $index:expr) => { asset_type: $asset_type:expr, quantity: $quantity:expr }) $(,$x:tt)*]) => {
        let asset = $state.asset($tracker, $index)
            .expect(&format!("Cannot read Asset from {}:{}:{}", $state.shard_id(), $tracker, $index))
            .expect(&format!("Asset for {}:{}:{} not exist", $state.shard_id(), $tracker, $index));
        assert_eq!(&$asset_type, asset.asset_type());
        assert_eq!($quantity, asset.quantity());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(asset: ($tracker:expr, $index:expr) => { asset_type: $asset_type:expr, quantity: $quantity:expr, lock_script_hash: $lock_script_hash:expr }) $(,$x:tt)*]) => {
        let asset = $state.asset($tracker, $index)
            .expect(&format!("Cannot read Asset from {}:{}:{}", $state.shard_id(), $tracker, $index))
            .expect(&format!("Asset for {}:{}:{} not exist", $state.shard_id(), $tracker, $index));
        assert_eq!(&$asset_type, asset.asset_type());
        assert_eq!($quantity, asset.quantity());
        assert_eq!(&$lock_script_hash, asset.lock_script_hash());

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(asset: ($tracker:expr, $index:expr)) $(,$x:tt)*]) => {
        assert_eq!(Ok(None), $state.asset($tracker, $index));

        check_shard_level_state!($state, [$($x),*]);
    };
    ($state:expr, [(text: ($tracker:expr) => { content: $content: expr}) $(,$x:tt)*]) => {
        let stored_text = $state.text($tracker)
            .expect(&format!("Cannot read Text from {}:{}", $state.shard_id(), $tracker))
            .expect(&format!("Text for {}:{} not exist", $state.shard_id(), $tracker));

        assert_eq!($content, stored_text.content());

        check_shard_level_state!($state, [$($x), *])
    };
}
