// Copyright 2020 Kodebox, Inc.
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
use crate::{ModuleDatum, ModuleDatumAddress};
use merkle_trie::{Result as TrieResult, Trie, TrieMut};
use std::cell::RefMut;

#[derive(Clone)]
pub struct ModuleCache {
    data: WriteBack<ModuleDatum>,
}

impl ModuleCache {
    pub fn new(module_data: impl Iterator<Item = (ModuleDatumAddress, ModuleDatum)>) -> Self {
        Self {
            data: WriteBack::new_with_iter(module_data),
        }
    }

    pub fn checkpoint(&mut self) {
        self.data.checkpoint();
    }

    pub fn discard_checkpoint(&mut self) {
        self.data.discard_checkpoint();
    }

    pub fn revert_to_checkpoint(&mut self) {
        self.data.revert_to_checkpoint();
    }

    pub fn commit(&mut self, trie: &mut dyn TrieMut) -> TrieResult<()> {
        self.data.commit(trie)?;
        Ok(())
    }

    pub fn create_module_datum<F>(&self, a: &ModuleDatumAddress, f: F) -> TrieResult<ModuleDatum>
    where
        F: FnOnce() -> ModuleDatum, {
        self.data.create(a, f)
    }

    pub fn has(&self, a: &ModuleDatumAddress, db: &dyn Trie) -> TrieResult<bool> {
        self.data.has(a, db)
    }

    pub fn module_datum(&self, a: &ModuleDatumAddress, db: &dyn Trie) -> TrieResult<Option<ModuleDatum>> {
        self.data.get(a, db)
    }

    pub fn module_datum_mut(&self, a: &ModuleDatumAddress, db: &dyn Trie) -> TrieResult<RefMut<'_, ModuleDatum>> {
        self.data.get_mut(a, db)
    }

    pub fn remove_module_datum(&self, a: &ModuleDatumAddress) {
        self.data.remove(a);
    }

    pub fn cached_module_datum(&self) -> Vec<(usize, ModuleDatumAddress, Option<ModuleDatum>)> {
        self.data.items()
    }
}

impl Default for ModuleCache {
    fn default() -> Self {
        Self::new(::std::iter::empty())
    }
}
