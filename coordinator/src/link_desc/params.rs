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

use super::ModuleSetup;
use crate::{desc_common::params::Merger, LinkDesc};
use anyhow::Context as _;
use std::collections::BTreeMap;

impl LinkDesc {
    pub fn merge_params(&mut self, params: &BTreeMap<String, String>) -> anyhow::Result<()> {
        let mut merged_params = self.param_defaults.clone();
        merged_params.append(&mut params.clone());

        let merger = Merger::new(&merged_params);

        for (name, setup) in self.modules.iter_mut() {
            setup.merge_params(&merger).with_context(|| format!("module: {}", name))?;
        }
        Ok(())
    }
}

impl ModuleSetup {
    fn merge_params(&mut self, merger: &Merger) -> anyhow::Result<()> {
        for (export, cons) in self.exports.iter_mut() {
            cons.args.merge_params(merger).with_context(|| format!("exports > {} = {}", export, cons.name))?;
        }
        self.init_config.merge_params(merger).context("init-config")?;
        Ok(())
    }
}
