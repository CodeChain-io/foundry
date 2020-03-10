// Copyright 2018 Kodebox, Inc.
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

use crate::CacheableItem;
use lru_cache::LruCache as LruCacheImpl;

#[derive(Clone)]
pub struct LruCache<Item: CacheableItem> {
    cache: LruCacheImpl<Item::Address, Item>,
}

impl<Item: CacheableItem> LruCache<Item> {
    pub fn new(capacity: usize) -> Self {
        Self {
            cache: LruCacheImpl::new(capacity),
        }
    }

    pub fn clear(&mut self) {
        self.cache.clear()
    }

    /// Returns an iterator over the cache's key-value pairs in least- to most-recently-used order.
    pub fn iter(&self) -> impl Iterator<Item = (&Item::Address, &Item)> {
        self.cache.iter()
    }

    pub fn cloned_iter<'a>(&'a self) -> impl Iterator<Item = (Item::Address, Item)> + 'a {
        self.cache.iter().map(|(addr, value)| (*addr, value.clone()))
    }

    pub fn insert(&mut self, k: Item::Address, v: Item) -> Option<Item> {
        self.cache.insert(k, v)
    }

    pub fn remove(&mut self, k: &Item::Address) -> Option<Item> {
        self.cache.remove(&k)
    }
}
