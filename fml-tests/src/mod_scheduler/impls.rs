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

use super::get_context;
use crate::services::*;
use fml::*;

#[fml_macro::service_impl(Schedule, TraitHolder)]
pub struct MySchedule {
    pub handle: HandleInstance,
}

impl Schedule for MySchedule {
    fn get(&self) -> AvailiableMap {
        let mut avail = get_context().custom.lock.lock().unwrap();
        while !*avail {
            avail = get_context().custom.cvar.wait(avail).unwrap()
        }
        *avail = false;
        get_context().custom.map.lock().unwrap().clone()
    }

    fn set(&self, s: AvailiableMap) {
        *get_context().custom.map.lock().unwrap() = s;
        *get_context().custom.lock.lock().unwrap() = true;
        get_context().custom.cvar.notify_one();
    }
}
