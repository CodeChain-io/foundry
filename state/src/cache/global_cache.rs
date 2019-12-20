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

use super::lru_cache::LruCache;
use super::TopCache;
use crate::{Account, ActionData, Metadata, RegularAccount, Text};

pub struct GlobalCache {
    account: LruCache<Account>,
    regular_account: LruCache<RegularAccount>,
    metadata: LruCache<Metadata>,
    text: LruCache<Text>,
    action_data: LruCache<ActionData>,
}

impl GlobalCache {
    pub fn new(account: usize, regular_account: usize, text: usize, action_data: usize) -> Self {
        Self {
            account: LruCache::new(account),
            regular_account: LruCache::new(regular_account),
            metadata: LruCache::new(1),
            text: LruCache::new(text),
            action_data: LruCache::new(action_data),
        }
    }

    pub fn top_cache(&self) -> TopCache {
        TopCache::new(
            self.account.iter().map(|(addr, item)| (*addr, item.clone())),
            self.regular_account.iter().map(|(addr, item)| (*addr, item.clone())),
            self.metadata.iter().map(|(addr, item)| (*addr, item.clone())),
            self.text.iter().map(|(addr, item)| (*addr, item.clone())),
            self.action_data.iter().map(|(addr, item)| (*addr, item.clone())),
        )
    }

    pub fn override_cache(&mut self, top_cache: &TopCache) {
        self.clear();

        for (addr, item) in top_cache.cached_accounts().into_iter() {
            match item {
                Some(item) => self.account.insert(addr, item),
                None => self.account.remove(&addr),
            };
        }
        for (addr, item) in top_cache.cached_regular_accounts().into_iter() {
            match item {
                Some(item) => self.regular_account.insert(addr, item),
                None => self.regular_account.remove(&addr),
            };
        }
        for (addr, item) in top_cache.cached_metadata().into_iter() {
            match item {
                Some(item) => self.metadata.insert(addr, item),
                None => self.metadata.remove(&addr),
            };
        }
        for (addr, item) in top_cache.cached_texts().into_iter() {
            match item {
                Some(item) => self.text.insert(addr, item),
                None => self.text.remove(&addr),
            };
        }
        for (addr, item) in top_cache.cached_action_data().into_iter() {
            match item {
                Some(item) => self.action_data.insert(addr, item),
                None => self.action_data.remove(&addr),
            };
        }
    }

    pub fn clear(&mut self) {
        self.account.clear();
        self.regular_account.clear();
        self.metadata.clear();
        self.text.clear();
        self.action_data.clear();
    }
}

impl Default for GlobalCache {
    fn default() -> Self {
        // FIXME: Set the right number
        const N_ACCOUNT: usize = 100;
        const N_REGULAR_ACCOUNT: usize = 100;
        const N_TEXT: usize = 100;
        const N_ACTION_DATA: usize = 10;
        Self::new(N_ACCOUNT, N_REGULAR_ACCOUNT, N_TEXT, N_ACTION_DATA)
    }
}

impl Clone for GlobalCache {
    fn clone(&self) -> Self {
        Self {
            account: self.account.clone(),
            regular_account: self.regular_account.clone(),
            metadata: self.metadata.clone(),
            text: self.text.clone(),
            action_data: self.action_data.clone(),
        }
    }
}
