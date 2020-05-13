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

use crate::services::*;
use fml::*;

#[fml_macro::service_impl(HelloFactory, TraitHolder)]
pub struct Factory {
    pub handle: HandleInstance,
}

impl HelloFactory for Factory {
    fn create(&self, name: &String) -> Box<dyn HelloRobot> {
        Box::new(Robot {
            handle: Default::default(),
            name: name.to_string(),
        })
    }
}

#[fml_macro::service_impl(HelloRobot, TraitHolder)]
pub struct Robot {
    pub handle: HandleInstance,
    pub name: String,
}

impl HelloRobot for Robot {
    fn hello(&self, flag: i32) -> String {
        format!("{}{}", self.name, flag)
    }
}
