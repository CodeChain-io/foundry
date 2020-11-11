// Copyright 2020 Kodebox, Inc.
// This file is part of CodeChain.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as
// published by the Free Software Foundation, either version 3 of the
// License, or (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU Affero General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

use crate::desc_common::GlobalName;
use crate::{app_desc::Namespaced, LinkDesc};
use anyhow::bail;

impl LinkDesc {
    pub fn validate(&self) -> anyhow::Result<()> {
        self.sandboxer_specified()?;
        self.module_imports_are_valid()?;

        Ok(())
    }

    fn sandboxer_specified(&self) -> anyhow::Result<()> {
        if !self.default_sandboxer.is_empty() {
            return Ok(())
        }

        let modules_without_sandboxer: Vec<_> = self
            .modules
            .iter()
            .filter_map(|(module, setup)| {
                if setup.sandboxer.is_empty() {
                    Some(&**module as &str)
                } else {
                    None
                }
            })
            .collect();

        if modules_without_sandboxer.is_empty() {
            return Ok(())
        }

        bail!("No sandboxer is specified for modules: {}", modules_without_sandboxer.join(", "))
    }

    fn module_imports_are_valid(&self) -> anyhow::Result<()> {
        for (module, setup) in self.modules.iter() {
            self.imports_are_valid(&format!("A module, '{}'", module), &setup.imports)?;
        }
        Ok(())
    }

    fn imports_are_valid(&self, importer: &str, imports: &Namespaced<GlobalName>) -> anyhow::Result<()> {
        for (_to, from) in imports.iter() {
            let module = from.module();
            match self.modules.get(from.module()) {
                Some(setup) => {
                    let name = from.name();
                    if !setup.exports.contains_key(name) {
                        bail!("{} imports non-existing service '{}' from '{}'", importer, name, module)
                    }
                }
                None => bail!("{} imports from non-existing module: {}", importer, module),
            }
        }

        Ok(())
    }
}
