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

use super::AppDesc;
use crate::app_desc::{GlobalName, Namespaced};
use anyhow::bail;

impl AppDesc {
    pub fn validate(&self) -> anyhow::Result<()> {
        self.sandboxer_specified()?;
        self.tx_owners_are_valid()?;
        self.host_imports_are_valid()?;
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

    fn tx_owners_are_valid(&self) -> anyhow::Result<()> {
        let invalid_owners: Vec<(&str, &str)> = self
            .transactions
            .iter()
            .filter_map(|(tx_type, tx_owner)| {
                if self.modules.contains_key(tx_owner) {
                    None
                } else {
                    Some((tx_type as &str, &**tx_owner as &str))
                }
            })
            .collect();

        if !(invalid_owners.is_empty()) {
            Ok(())
        } else {
            bail!(
                "No such owners for transactions: {}",
                invalid_owners
                    .into_iter()
                    .map(|(tx_type, tx_owner)| format!("{}: {}", tx_type, tx_owner))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    }

    fn host_imports_are_valid(&self) -> anyhow::Result<()> {
        self.imports_are_valid("The host", &self.host.imports)
    }

    fn module_imports_are_valid(&self) -> anyhow::Result<()> {
        for (module, setup) in self.modules.iter() {
            self.imports_are_valid(&format!("A module, '{}'", module), &setup.imports)?;
        }
        Ok(())
    }

    fn imports_are_valid(&self, importer: &str, imports: &Namespaced<GlobalName>) -> anyhow::Result<()> {
        for (to, from) in imports.iter() {
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
