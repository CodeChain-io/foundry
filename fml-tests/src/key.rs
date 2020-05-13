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

use fml::queue::Queue;
use fml::InstanceKey;
use once_cell::sync::OnceCell;

#[cfg(feature = "single_process")]
mod m {
    use super::*;
    use std::collections::VecDeque;
    use std::sync::Mutex;

    static POOL: OnceCell<Mutex<VecDeque<InstanceKey>>> = OnceCell::new();

    pub fn create_instance() -> InstanceKey {
        let x = POOL
            .get_or_init(|| {
                let mut x = VecDeque::new();
                for i in 1..5000 {
                    x.push_back(i);
                }
                Mutex::new(x)
            })
            .lock()
            .unwrap()
            .pop_front()
            .unwrap();
        x
    }

    pub fn return_instance(key: InstanceKey) {
        POOL.get().unwrap().lock().unwrap().push_back(key);
    }
}

#[cfg(not(feature = "single_process"))]
mod m {
    use super::*;

    pub fn create_instance() -> InstanceKey {
        1
    }

    pub fn return_instance(_key: InstanceKey) {}
}

pub use m::{create_instance, return_instance};

// FML tests require a lot of OS resources. We regulate the maximum number of parallel tests.

static TEST_KEYS: OnceCell<Queue<i32>> = OnceCell::new();

pub fn start_test() -> i32 {
    TEST_KEYS
        .get_or_init(|| {
            let q = Queue::new(4);
            for i in 0..4 {
                q.push(i)
            }
            q
        })
        .pop(None)
        .unwrap()
}

pub fn end_test(k: i32) {
    TEST_KEYS.get().unwrap().push(k)
}
