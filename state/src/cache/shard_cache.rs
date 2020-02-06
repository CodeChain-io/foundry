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

use super::WriteBack;
use crate::{ShardText, ShardTextAddress};
use merkle_trie::{Result as TrieResult, Trie, TrieMut};
use std::cell::RefMut;

pub struct ShardCache {
    text: WriteBack<ShardText>,
}

impl ShardCache {
    pub fn new(shard_texts: impl Iterator<Item = (ShardTextAddress, ShardText)>) -> Self {
        Self {
            text: WriteBack::new_with_iter(shard_texts),
        }
    }

    pub fn checkpoint(&mut self) {
        self.text.checkpoint();
    }

    pub fn discard_checkpoint(&mut self) {
        self.text.discard_checkpoint();
    }

    pub fn revert_to_checkpoint(&mut self) {
        self.text.revert_to_checkpoint();
    }

    pub fn commit(&mut self, trie: &mut dyn TrieMut) -> TrieResult<()> {
        self.text.commit(trie)?;
        Ok(())
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

    pub fn cached_shard_text(&self) -> Vec<(usize, ShardTextAddress, Option<ShardText>)> {
        self.text.items()
    }
}

impl Clone for ShardCache {
    fn clone(&self) -> Self {
        Self {
            text: self.text.clone(),
        }
    }
}

impl Default for ShardCache {
    fn default() -> Self {
        Self::new(::std::iter::empty())
    }
}
