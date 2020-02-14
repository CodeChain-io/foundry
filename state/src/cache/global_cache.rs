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
use crate::{Account, ActionData, Metadata, Shard, ShardText};
use ctypes::ShardId;
use std::collections::{HashMap, HashSet};

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

    pub fn override_cache(&mut self, top_cache: &TopCache, shard_caches: &HashMap<ShardId, ShardCache>) {
        self.clear();

        for (addr, item) in top_cache.cached_accounts().into_iter() {
            match item {
                Some(item) => self.account.insert(addr, item),
                None => self.account.remove(&addr),
            };
        }
        for (addr, item) in top_cache.cached_metadata().into_iter() {
            match item {
                Some(item) => self.metadata.insert(addr, item),
                None => self.metadata.remove(&addr),
            };
        }
        for (addr, item) in top_cache.cached_shards().into_iter() {
            match item {
                Some(item) => self.shard.insert(addr, item),
                None => self.shard.remove(&addr),
            };
        }
        for (addr, item) in top_cache.cached_action_data().into_iter() {
            match item {
                Some(item) => self.action_data.insert(addr, item),
                None => self.action_data.remove(&addr),
            };
        }

        let mut cached_shard_texts: Vec<_> =
            shard_caches.iter().flat_map(|(_, shard_cache)| shard_cache.cached_shard_text().into_iter()).collect();
        cached_shard_texts.sort_unstable_by(|lhs, rhs| lhs.0.cmp(&rhs.0));
        for (_, addr, item) in cached_shard_texts.into_iter() {
            match item {
                Some(item) => self.shard_text.insert(addr, item),
                None => self.shard_text.remove(&addr),
            };
        }
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

impl Clone for GlobalCache {
    fn clone(&self) -> Self {
        Self {
            account: self.account.clone(),
            metadata: self.metadata.clone(),
            shard: self.shard.clone(),
            action_data: self.action_data.clone(),

            shard_text: self.shard_text.clone(),
        }
    }
}
