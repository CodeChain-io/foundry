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

use std::sync::Arc;

use crate::sandbox::{Result, Sandbox, Sandboxer, SANDBOXERS};
use std::path::Path;

#[distributed_slice(SANDBOXERS)]
fn builtin_loader() -> Arc<dyn Sandboxer> {
    Arc::new(BuiltinLoader {})
}

struct BuiltinLoader {}

impl Sandboxer for BuiltinLoader {
    fn id(&self) -> &'static str {
        "builtin"
    }

    fn supported_module_types(&self) -> &'static [&'static str] {
        &["builtin"]
    }

    fn load(
        &self,
        path: &dyn AsRef<Path>,
        init: &dyn erased_serde::Serialize,
        exports: &[(&str, &dyn erased_serde::Serialize)],
    ) -> Result<Arc<dyn Sandbox>> {
    }
}
