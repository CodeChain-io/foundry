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

use crate::app_desc::{AppDesc, HostSetup, ModuleSetup};
use crate::values::Value;
use anyhow::Context as _;
use handlebars::{no_escape, Context, Handlebars, RenderContext, Renderable, Template};
use std::collections::BTreeMap;

pub use handlebars::TemplateRenderError;
pub use serde_yaml::Error as YamlError;

#[allow(dead_code)]
struct Merger<'reg> {
    registry: Handlebars<'reg>,
    context: Context,
}

#[allow(dead_code)]
impl<'reg> Merger<'reg> {
    fn new(params: &BTreeMap<String, String>) -> Merger<'reg> {
        let mut registry = Handlebars::new();
        registry.register_escape_fn(no_escape);
        registry.set_strict_mode(true);

        Merger {
            registry,
            context: Context::wraps(params).unwrap(),
        }
    }

    fn merge(&self, s: &str) -> Result<String, TemplateRenderError> {
        let template = Template::compile(s)?;
        let mut render_context = RenderContext::new(None);

        Ok(template.renders(&self.registry, &self.context, &mut render_context)?)
    }
}

#[allow(dead_code)]
impl AppDesc {
    pub fn merge_params(&mut self, params: &BTreeMap<String, String>) -> anyhow::Result<()> {
        let mut merged_params = self.param_defaults.clone();
        merged_params.append(&mut params.clone());

        let merger = Merger::new(&merged_params);

        for (name, setup) in self.modules.iter_mut() {
            setup.merge_params(&merger).with_context(|| format!("module: {}", name))?;
        }
        self.host.merge_params(&merger)?;

        Ok(())
    }
}

impl ModuleSetup {
    fn merge_params(&mut self, merger: &Merger) -> anyhow::Result<()> {
        for (export, cons) in self.exports.iter_mut() {
            cons.args.merge_params(merger).with_context(|| format!("exports > {} = {}", export, cons.name))?;
        }
        for (config, value) in self.init_config.iter_mut() {
            value.merge_params(merger).with_context(|| format!("init-config > {}", config))?;
        }
        for (config, value) in self.genesis_config.iter_mut() {
            value.merge_params(merger).with_context(|| format!("genesis-config > {}", config))?;
        }
        Ok(())
    }
}

impl HostSetup {
    fn merge_params(&mut self, merger: &Merger) -> anyhow::Result<()> {
        for (export, cons) in self.exports.iter_mut() {
            cons.args.merge_params(merger).with_context(|| format!("host > exports > {} = {}", export, cons.name))?;
        }
        for (config, value) in self.init_config.iter_mut() {
            value.merge_params(merger).with_context(|| format!("host > init-config > {}", config))?;
        }
        for (config, value) in self.genesis_config.iter_mut() {
            value.merge_params(merger).with_context(|| format!("host > genesis-config > {}", config))?;
        }
        Ok(())
    }
}

impl Value {
    fn merge_params(&mut self, merger: &Merger) -> anyhow::Result<()> {
        match self {
            Self::String(s) => {
                *self = serde_yaml::from_str::<Value>(&merger.merge(s)?)?;
            }
            Self::List(list) => {
                for v in list {
                    v.merge_params(merger)?
                }
            }
            Self::Map(map) => {
                for v in map.values_mut() {
                    v.merge_params(merger)?
                }
            }
            _ => {}
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Merger;
    use crate::values::Value;

    #[test]
    fn merge_into_string_value() {
        let params = vec![("hello".to_owned(), "world".to_owned())].into_iter().collect();
        let merger = Merger::new(&params);
        let mut value = Value::String("=={{hello}}==".to_owned());
        value.merge_params(&merger).unwrap();
        assert_eq!(value, Value::String("==world==".to_owned()))
    }

    #[test]
    fn merge_into_non_string_value() {
        let params = vec![("hello".to_owned(), "world".to_owned())].into_iter().collect();
        let merger = Merger::new(&params);
        let mut value = Value::Int(123);
        value.merge_params(&merger).unwrap();
        assert_eq!(value, Value::Int(123))
    }

    #[test]
    fn merge_into_value_in_map() {
        let params = vec![("hello".to_owned(), "world".to_owned())].into_iter().collect();
        let merger = Merger::new(&params);
        let mut value = Value::Map(
            vec![(String::from("1"), Value::Int(1)), (String::from("2"), Value::String(String::from("=={{hello}}==")))]
                .into_iter()
                .collect(),
        );

        value.merge_params(&merger).unwrap();

        assert_eq!(
            value,
            Value::Map(
                vec![(String::from("1"), Value::Int(1)), (String::from("2"), Value::String(String::from("==world==")))]
                    .into_iter()
                    .collect()
            )
        );
    }

    #[test]
    fn merge_into_value_in_list() {
        let params = vec![("hello".to_owned(), "world".to_owned())].into_iter().collect();
        let merger = Merger::new(&params);
        let mut value = Value::List(vec![Value::Int(1), Value::String(String::from("=={{hello}}=="))]);

        value.merge_params(&merger).unwrap();

        assert_eq!(value, Value::List(vec![Value::Int(1), Value::String(String::from("==world=="))]));
    }
}
