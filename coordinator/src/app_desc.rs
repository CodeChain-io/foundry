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
pub use crate::desc_common::{Constructor, GlobalName, LocalName, Namespaced, SimpleName};
pub use engine::Engine;
use foundry_hex::Hex;
pub use genesis::Genesis;
use primitives::H256;
use serde::Deserialize;
use std::collections::HashMap;
use std::fmt::Debug;
pub use tendermint::TendermintParams;

mod engine;
mod genesis;
pub(self) mod params;
mod tendermint;
pub(self) mod validator;

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AppDesc {
    // keyed with Name rather than module hash to allow for multiple instances of single module
    pub modules: HashMap<SimpleName, ModuleSetup>,
    #[serde(default)]
    pub host: HostSetup,
    #[serde(default)]
    pub transactions: Namespaced<SimpleName>,
    #[serde(default)]
    pub param_defaults: Namespaced<String>,
}

#[allow(clippy::should_implement_trait)]
impl AppDesc {
    pub fn from_str(s: &str) -> anyhow::Result<AppDesc> {
        let app_desc: AppDesc = toml::from_str(s)?;
        app_desc.validate()?;

        Ok(app_desc)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ModuleSetup {
    pub hash: Hex<H256>,
    /// List of export names expected to hold the required services.
    /// Then the module will receive imports for `@tx/<transaction-type>/<export-name>`s.
    /// It is mainly intended for modules providing `TxSorter` service.
    #[serde(default)]
    pub transactions: Vec<LocalName>,
    #[serde(default)]
    pub genesis_config: Value,
    #[serde(default)]
    pub tags: HashMap<String, Value>,
}

#[derive(Deserialize, Default, Debug)]
pub struct HostSetup {
    #[serde(default)]
    pub genesis_config: Namespaced<Value>,
    #[serde(default)]
    pub engine: Engine,
    #[serde(default)]
    pub genesis: Genesis,
}

#[cfg(test)]
mod tests {
    use crate::app_desc::AppDesc;
    use unindent::unindent;

    #[test]
    fn load_essentials() {
        let source = unindent(
            r#"
[modules.awesome-module]
hash = "1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef"
transactions = ["has-seq"]

[modules.awesome-module.init-config.test1]
key1 = 1
key2 = "sdfsdaf"

[host]

[transactions]
great-tx = "awesome-module"

[param-defaults]
num-threads = "10"
            "#,
        );
        let _: AppDesc = toml::from_str(&source).unwrap();
    }
}
