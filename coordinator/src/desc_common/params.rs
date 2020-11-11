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

use crate::values::TOMLValueDeserializer;
use crate::values::Value;
use handlebars::{no_escape, Context, Handlebars, TemplateRenderError};
use std::collections::BTreeMap;

/// You use Handlebars style template in app descriptor.
///   some-config = "{{other-variable}}"
/// By starting "@" you can express map or array using template.
///   some-config-array = "@[ {{variable1}}, {{variable2}} ]"
///   some-config-map = "@[ \"key\" = {{value-variable}} ]"
/// If you want to start a string value please use "@@"
///   some-config = "@@start from @"
pub struct Merger<'reg> {
    registry: Handlebars<'reg>,
    context: Context,
}

#[allow(dead_code)]
impl<'reg> Merger<'reg> {
    pub(crate) fn new(params: &BTreeMap<String, String>) -> Merger<'reg> {
        let mut registry = Handlebars::new();
        registry.register_escape_fn(no_escape);
        registry.set_strict_mode(true);

        Merger {
            registry,
            context: Context::wraps(params).unwrap(),
        }
    }

    fn merge(&self, s: &str) -> Result<String, TemplateRenderError> {
        self.registry.render_template_with_context(s, &self.context)
    }
}

impl Value {
    pub(crate) fn merge_params(&mut self, merger: &Merger) -> anyhow::Result<()> {
        match self {
            Self::String(s) => {
                if s.starts_with("@@") {
                    // Change @@ to @ and type is string
                    let merged = merger.merge(&s[1..])?;
                    *self = Value::String(merged);
                } else if s.starts_with('@') {
                    // Remove @ and type is anything
                    let merged = merger.merge(&s[1..])?;
                    *self = TOMLValueDeserializer::deserialize(&merged)?;
                } else {
                    // Type is string
                    let merged = merger.merge(s)?;
                    *self = Value::String(merged);
                }
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

    #[test]
    fn merge_at() {
        let params = vec![("hello".to_owned(), "world".to_owned())].into_iter().collect();
        let merger = Merger::new(&params);
        let mut value = Value::String("@\"=={{hello}}==\"".to_owned());
        value.merge_params(&merger).unwrap();
        assert_eq!(value, Value::String("==world==".to_owned()))
    }

    #[test]
    fn merge_at_array() {
        let params = vec![("hello".to_owned(), "world".to_owned())].into_iter().collect();
        let merger = Merger::new(&params);
        let mut value = Value::String("@[\"{{hello}}\", \"{{hello}}\"]".to_owned());
        value.merge_params(&merger).unwrap();
        assert_eq!(value, Value::List(vec![Value::String("world".to_owned()), Value::String("world".to_owned())]))
    }

    #[test]
    fn merge_at_map() {
        let params = vec![("hello".to_owned(), "world".to_owned())].into_iter().collect();
        let merger = Merger::new(&params);
        let mut value = Value::String("@{ \"key\" = \"{{hello}}\" }".to_owned());
        value.merge_params(&merger).unwrap();
        assert_eq!(
            value,
            Value::Map(vec![("key".to_string(), Value::String("world".to_string())),].into_iter().collect())
        )
    }

    #[test]
    fn merge_atat() {
        let params = vec![("hello".to_owned(), "world".to_owned())].into_iter().collect();
        let merger = Merger::new(&params);
        let mut value = Value::String("@@=={{hello}}==".to_owned());
        value.merge_params(&merger).unwrap();
        assert_eq!(value, Value::String("@==world==".to_owned()))
    }
}
