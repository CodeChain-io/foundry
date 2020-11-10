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

use super::values::Value;
use crate::desc_common::{Constructor, GlobalName, Namespaced, SimpleName};
use serde::Deserialize;
use std::collections::HashMap;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct LinkDesc {
    /// The ID of the default `Sandboxer` to be used when no `Sandboxer` is specified for modules.
    #[serde(default)]
    pub default_sandboxer: String,
    pub modules: HashMap<SimpleName, ModuleSetup>,
    #[serde(default)]
    pub param_defaults: Namespaced<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ModuleSetup {
    #[serde(default)]
    pub sandboxer: String,
    #[serde(default)]
    pub exports: Namespaced<Constructor>,
    #[serde(default)]
    pub imports: Namespaced<GlobalName>,
    #[serde(default)]
    pub init_config: Value,
}

#[allow(clippy::should_implement_trait)]
impl LinkDesc {
    pub fn from_str(s: &str) -> anyhow::Result<LinkDesc> {
        let link_desc: LinkDesc = toml::from_str(s)?;
        Ok(link_desc)
    }
}
