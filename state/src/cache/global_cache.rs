// Copyright 2018-2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use super::lru_cache::LruCache;
use super::{ModuleCache, TopCache};
use crate::{ActionData, Metadata, Module, ModuleDatum};
use crate::{CacheableItem, ModuleDatumAddress};
use ctypes::StorageId;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct GlobalCache {
    metadata: LruCache<Metadata>,
    module: LruCache<Module>,
    action_data: LruCache<ActionData>,

    module_data: LruCache<ModuleDatum>,
}

impl GlobalCache {
    pub fn new(_account: usize, module: usize, action_data: usize, module_data: usize) -> Self {
        Self {
            metadata: LruCache::new(1),
            module: LruCache::new(module),
            action_data: LruCache::new(action_data),

            module_data: LruCache::new(module_data),
        }
    }

    pub fn top_cache(&self) -> TopCache {
        TopCache::new(self.metadata.cloned_iter(), self.module.cloned_iter(), self.action_data.cloned_iter())
    }

    fn module_cache(&self, storage_id: StorageId) -> ModuleCache {
        ModuleCache::new(
            self.module_data
                .iter()
                .filter(|(addr, _)| addr.storage_id() == storage_id)
                .map(|(addr, item)| (*addr, item.clone())),
        )
    }

    fn storage_ids(&self) -> HashSet<StorageId> {
        self.module_data.iter().map(|(addr, _)| addr.storage_id()).collect()
    }

    pub fn module_caches(&self) -> HashMap<StorageId, ModuleCache> {
        self.storage_ids().into_iter().map(|storage_id| (storage_id, self.module_cache(storage_id))).collect()
    }

    fn drain_cacheable_into_lru_cache<T: CacheableItem>(from: Vec<(T::Address, Option<T>)>, to: &mut LruCache<T>) {
        from.into_iter().for_each(|(addr, item)| {
            match item {
                Some(item) => to.insert(addr, item),
                None => to.remove(&addr),
            };
        })
    }

    pub fn override_cache(
        &mut self,
        top_cache: &TopCache,
        module_data: Vec<(ModuleDatumAddress, Option<ModuleDatum>)>,
    ) {
        self.clear();

        Self::drain_cacheable_into_lru_cache(top_cache.cached_metadata(), &mut self.metadata);
        Self::drain_cacheable_into_lru_cache(top_cache.cached_action_data(), &mut self.action_data);
        Self::drain_cacheable_into_lru_cache(top_cache.cached_modules(), &mut self.module);
        Self::drain_cacheable_into_lru_cache(module_data, &mut self.module_data);
    }

    pub fn clear(&mut self) {
        self.metadata.clear();
        self.module.clear();
        self.action_data.clear();
        self.module_data.clear();
    }
}

impl Default for GlobalCache {
    fn default() -> Self {
        // FIXME: Set the right number
        const N_ACCOUNT: usize = 100;
        const N_MODULE: usize = 10;
        const N_ACTION_DATA: usize = 10;
        const N_MODULE_DATA: usize = 1000;
        Self::new(N_ACCOUNT, N_MODULE, N_ACTION_DATA, N_MODULE_DATA)
    }
}
