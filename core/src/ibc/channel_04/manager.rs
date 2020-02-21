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

use crate::ibc;
use crate::ibc::Identifier;

pub struct Manager<'a> {
    ctx: &'a mut dyn ibc::Context,
}

/// Temporary dummy functions for port05
fn port05_generate() -> Identifier {
    "".to_owned()
}

#[allow(unused_variables, dead_code)]
fn port05_authenticate(key: Identifier) -> bool {
    true
}

/// For all functions, there are some difference from the spec.
/// 1. They take only single Identifier as connection, since we won't consider the `hop`.
/// 2. They take no ports : All ports will be considered as DEFAULT_PORT.
impl<'a> Manager<'a> {
    pub fn new(ctx: &'a mut dyn ibc::Context) -> Self {
        Manager {
            ctx,
        }
    }
}
