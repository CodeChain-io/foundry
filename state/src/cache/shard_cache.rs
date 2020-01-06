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

use super::WriteBack;
use crate::{AssetScheme, AssetSchemeAddress, OwnedAsset, OwnedAssetAddress, ShardText, ShardTextAddress};
use cmerkle::{Result as TrieResult, Trie, TrieMut};
use std::cell::RefMut;

pub struct ShardCache {
    asset_scheme: WriteBack<AssetScheme>,
    asset: WriteBack<OwnedAsset>,
    text: WriteBack<ShardText>,
}

impl ShardCache {
    pub fn new(
        asset_schemes: impl Iterator<Item = (AssetSchemeAddress, AssetScheme)>,
        assets: impl Iterator<Item = (OwnedAssetAddress, OwnedAsset)>,
        shard_texts: impl Iterator<Item = (ShardTextAddress, ShardText)>,
    ) -> Self {
        Self {
            asset_scheme: WriteBack::new_with_iter(asset_schemes),
            asset: WriteBack::new_with_iter(assets),
            text: WriteBack::new_with_iter(shard_texts),
        }
    }

    pub fn checkpoint(&mut self) {
        self.asset_scheme.checkpoint();
        self.asset.checkpoint();
        self.text.checkpoint();
    }

    pub fn discard_checkpoint(&mut self) {
        self.asset_scheme.discard_checkpoint();
        self.asset.discard_checkpoint();
        self.text.discard_checkpoint();
    }

    pub fn revert_to_checkpoint(&mut self) {
        self.asset_scheme.revert_to_checkpoint();
        self.asset.revert_to_checkpoint();
        self.text.revert_to_checkpoint();
    }

    pub fn commit(&mut self, trie: &mut dyn TrieMut) -> TrieResult<()> {
        self.asset_scheme.commit(trie)?;
        self.asset.commit(trie)?;
        self.text.commit(trie)?;
        Ok(())
    }

    pub fn asset_scheme(&self, a: &AssetSchemeAddress, db: &dyn Trie) -> TrieResult<Option<AssetScheme>> {
        self.asset_scheme.get(a, db)
    }

    pub fn asset_scheme_mut(&self, a: &AssetSchemeAddress, db: &dyn Trie) -> TrieResult<RefMut<'_, AssetScheme>> {
        self.asset_scheme.get_mut(a, db)
    }

    pub fn create_asset_scheme<F>(&self, a: &AssetSchemeAddress, f: F) -> TrieResult<AssetScheme>
    where
        F: FnOnce() -> AssetScheme, {
        self.asset_scheme.create(a, f)
    }

    pub fn asset(&self, a: &OwnedAssetAddress, db: &dyn Trie) -> TrieResult<Option<OwnedAsset>> {
        self.asset.get(a, db)
    }

    pub fn create_asset<F>(&self, a: &OwnedAssetAddress, f: F) -> TrieResult<OwnedAsset>
    where
        F: FnOnce() -> OwnedAsset, {
        self.asset.create(a, f)
    }

    pub fn remove_asset(&self, address: &OwnedAssetAddress) {
        self.asset.remove(address)
    }

    pub fn create_shard_text<F>(&self, a: &ShardTextAddress, f: F) -> TrieResult<ShardText>
    where
        F: FnOnce() -> ShardText, {
        self.text.create(a, f)
    }

    pub fn shard_text(&self, a: &ShardTextAddress, db: &dyn Trie) -> TrieResult<Option<ShardText>> {
        self.text.get(a, db)
    }

    pub fn shard_text_mut(&self, a: &ShardTextAddress, db: &dyn Trie) -> TrieResult<RefMut<'_, ShardText>> {
        self.text.get_mut(a, db)
    }

    pub fn remove_shard_text(&self, a: &ShardTextAddress) {
        self.text.remove(a);
    }

    pub fn cached_assets(&self) -> Vec<(usize, OwnedAssetAddress, Option<OwnedAsset>)> {
        self.asset.items()
    }

    pub fn cached_asset_schemes(&self) -> Vec<(usize, AssetSchemeAddress, Option<AssetScheme>)> {
        self.asset_scheme.items()
    }

    pub fn cached_shard_text(&self) -> Vec<(usize, ShardTextAddress, Option<ShardText>)> {
        self.text.items()
    }
}

impl Clone for ShardCache {
    fn clone(&self) -> Self {
        Self {
            asset_scheme: self.asset_scheme.clone(),
            asset: self.asset.clone(),
            text: self.text.clone(),
        }
    }
}

impl Default for ShardCache {
    fn default() -> Self {
        Self::new(::std::iter::empty(), ::std::iter::empty(), ::std::iter::empty())
    }
}
