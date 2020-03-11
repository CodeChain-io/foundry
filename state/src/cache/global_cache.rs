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

use super::lru_cache::LruCache;
use super::{ShardCache, TopCache};
use crate::CacheableItem;
use crate::{Account, ActionData, Metadata, Shard, ShardText};
use ctypes::ShardId;
use std::collections::{HashMap, HashSet};

#[derive(Clone)]
pub struct GlobalCache {
    account: LruCache<Account>,
    metadata: LruCache<Metadata>,
    shard: LruCache<Shard>,
    action_data: LruCache<ActionData>,

    shard_text: LruCache<ShardText>,
}

impl GlobalCache {
    pub fn new(account: usize, shard: usize, action_data: usize, shard_text: usize) -> Self {
        Self {
            account: LruCache::new(account),
            metadata: LruCache::new(1),
            shard: LruCache::new(shard),
            action_data: LruCache::new(action_data),

            shard_text: LruCache::new(shard_text),
        }
    }

    pub fn top_cache(&self) -> TopCache {
        TopCache::new(
            self.account.iter().map(|(addr, item)| (*addr, item.clone())),
            self.metadata.iter().map(|(addr, item)| (*addr, item.clone())),
            self.shard.iter().map(|(addr, item)| (*addr, item.clone())),
            self.action_data.iter().map(|(addr, item)| (*addr, item.clone())),
        )
    }

    fn shard_cache(&self, shard_id: ShardId) -> ShardCache {
        ShardCache::new(
            self.shard_text
                .iter()
                .filter(|(addr, _)| addr.shard_id() == shard_id)
                .map(|(addr, item)| (*addr, item.clone())),
        )
    }

    fn shard_ids(&self) -> HashSet<ShardId> {
        self.shard_text.iter().map(|(addr, _)| addr.shard_id()).collect()
    }

    pub fn shard_caches(&self) -> HashMap<ShardId, ShardCache> {
        self.shard_ids().into_iter().map(|shard_id| (shard_id, self.shard_cache(shard_id))).collect()
    }

    fn drain_cacheable_into_lru_cache<T: CacheableItem>(from: Vec<(T::Address, Option<T>)>, to: &mut LruCache<T>) {
        from.into_iter().for_each(|(addr, item)| {
            match item {
                Some(item) => to.insert(addr, item),
                None => to.remove(&addr),
            };
        })
    }

    pub fn override_cache(&mut self, top_cache: &TopCache, shard_caches: &HashMap<ShardId, ShardCache>) {
        self.clear();

        Self::drain_cacheable_into_lru_cache(top_cache.cached_accounts(), &mut self.account);
        Self::drain_cacheable_into_lru_cache(top_cache.cached_metadata(), &mut self.metadata);
        Self::drain_cacheable_into_lru_cache(top_cache.cached_shards(), &mut self.shard);
        Self::drain_cacheable_into_lru_cache(top_cache.cached_action_data(), &mut self.action_data);

        let mut cached_shard_texts: Vec<_> =
            shard_caches.iter().flat_map(|(_, shard_cache)| shard_cache.cached_shard_text().into_iter()).collect();
        cached_shard_texts.sort_unstable_by_key(|item| item.0);
        let cached_shard_texts: Vec<_> = cached_shard_texts.into_iter().map(|(_, addr, item)| (addr, item)).collect();
        Self::drain_cacheable_into_lru_cache(cached_shard_texts, &mut self.shard_text);
    }

    pub fn clear(&mut self) {
        self.account.clear();
        self.metadata.clear();
        self.shard.clear();
        self.action_data.clear();
        self.shard_text.clear();
    }
}

impl Default for GlobalCache {
    fn default() -> Self {
        // FIXME: Set the right number
        const N_ACCOUNT: usize = 100;
        const N_SHARD: usize = 100;
        const N_ACTION_DATA: usize = 10;
        const N_SHARD_TEXT: usize = 1000;
        Self::new(N_ACCOUNT, N_SHARD, N_ACTION_DATA, N_SHARD_TEXT)
    }
}
