// Copyright 2018, 2020 Kodebox, Inc.
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
use crate::{Account, ActionData, Metadata, MetadataAddress, Module, ModuleAddress};
use ckey::Address;
use merkle_trie::{Result as TrieResult, Trie, TrieMut};
use primitives::H256;
use std::cell::RefMut;

#[derive(Clone)]
pub struct TopCache {
    account: WriteBack<Account>,
    metadata: WriteBack<Metadata>,
    module: WriteBack<Module>,
    action_data: WriteBack<ActionData>,
}

impl TopCache {
    pub fn new(
        accounts: impl Iterator<Item = (Address, Account)>,
        metadata: impl Iterator<Item = (MetadataAddress, Metadata)>,
        modules: impl Iterator<Item = (ModuleAddress, Module)>,
        action_data: impl Iterator<Item = (H256, ActionData)>,
    ) -> Self {
        Self {
            account: WriteBack::new_with_iter(accounts),
            metadata: WriteBack::new_with_iter(metadata),
            module: WriteBack::new_with_iter(modules),
            action_data: WriteBack::new_with_iter(action_data),
        }
    }

    pub fn checkpoint(&mut self) {
        self.account.checkpoint();
        self.metadata.checkpoint();
        self.action_data.checkpoint();
    }

    pub fn discard_checkpoint(&mut self) {
        self.account.discard_checkpoint();
        self.metadata.discard_checkpoint();
        self.action_data.discard_checkpoint();
    }

    pub fn revert_to_checkpoint(&mut self) {
        self.account.revert_to_checkpoint();
        self.metadata.revert_to_checkpoint();
        self.action_data.revert_to_checkpoint();
    }

    pub fn commit<'db>(&mut self, trie: &mut (dyn TrieMut + 'db)) -> TrieResult<()> {
        self.account.commit(trie)?;
        self.metadata.commit(trie)?;
        self.action_data.commit(trie)?;
        Ok(())
    }

    pub fn account(&self, a: &Address, db: &dyn Trie) -> TrieResult<Option<Account>> {
        self.account.get(a, db)
    }

    pub fn account_mut(&self, a: &Address, db: &dyn Trie) -> TrieResult<RefMut<'_, Account>> {
        self.account.get_mut(a, db)
    }

    pub fn remove_account(&self, address: &Address) {
        self.account.remove(address)
    }

    pub fn metadata(&self, a: &MetadataAddress, db: &dyn Trie) -> TrieResult<Option<Metadata>> {
        self.metadata.get(a, db)
    }

    pub fn metadata_mut(&self, a: &MetadataAddress, db: &dyn Trie) -> TrieResult<RefMut<'_, Metadata>> {
        self.metadata.get_mut(a, db)
    }

    pub fn module(&self, a: &ModuleAddress, db: &dyn Trie) -> TrieResult<Option<Module>> {
        self.module.get(a, db)
    }

    pub fn module_mut(&self, a: &ModuleAddress, db: &dyn Trie) -> TrieResult<RefMut<'_, Module>> {
        self.module.get_mut(a, db)
    }

    pub fn action_data(&self, a: &H256, db: &dyn Trie) -> TrieResult<Option<ActionData>> {
        self.action_data.get(a, db)
    }

    pub fn action_data_mut(&self, a: &H256, db: &dyn Trie) -> TrieResult<RefMut<'_, ActionData>> {
        self.action_data.get_mut(a, db)
    }

    pub fn remove_action_data(&self, address: &H256) {
        self.action_data.remove(address)
    }

    pub fn cached_accounts(&self) -> Vec<(Address, Option<Account>)> {
        self.account.items_sorted_by_touched()
    }

    pub fn cached_metadata(&self) -> Vec<(MetadataAddress, Option<Metadata>)> {
        self.metadata.items_sorted_by_touched()
    }

    pub fn cached_action_data(&self) -> Vec<(H256, Option<ActionData>)> {
        self.action_data.items_sorted_by_touched()
    }

    pub fn cached_modules(&self) -> Vec<(ModuleAddress, Option<Module>)> {
        self.module.items_sorted_by_touched()
    }
}
