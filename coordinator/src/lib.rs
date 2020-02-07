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

pub mod context;

use self::context::Context;

/// The `Coordinator` encapsulates all the logic for a Foundry application.
///
/// It assembles modules and feeds them various events from the underlying
/// consensus engine.
pub struct Coordinator {}

pub struct Builder<C: Context> {
    _context: C,
}

impl<C: Context> Builder<C> {
    #[allow(dead_code)]
    fn new(context: C) -> Self {
        Builder {
            _context: context,
        }
    }

    #[allow(dead_code)]
    fn build(self) -> Coordinator {
        Coordinator {}
    }
}
