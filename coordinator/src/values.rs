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

use serde::de::{Error, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::collections::HashMap;
use std::fmt;

/// Generic value that may be specified in the app descriptor and module manifests.
#[derive(PartialEq, Debug)]
pub enum Value {
    Null,
    Int(i128),
    Bool(bool),
    String(String),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

impl Default for Value {
    fn default() -> Self {
        Value::Null
    }
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = Value;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("any YAML value")
            }

            fn visit_bool<E: Error>(self, b: bool) -> Result<Value, E> {
                Ok(Value::Bool(b))
            }

            fn visit_i64<E: Error>(self, i: i64) -> Result<Value, E> {
                Ok(Value::Int(i.into()))
            }

            fn visit_u64<E: Error>(self, u: u64) -> Result<Value, E> {
                Ok(Value::Int(u.into()))
            }

            fn visit_str<E: Error>(self, s: &str) -> Result<Value, E> {
                Ok(Value::String(s.to_owned()))
            }

            fn visit_string<E: Error>(self, s: String) -> Result<Value, E> {
                Ok(Value::String(s))
            }

            fn visit_none<E: Error>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Value, D::Error> {
                Deserialize::deserialize(deserializer)
            }

            fn visit_unit<E: Error>(self) -> Result<Value, E> {
                Ok(Value::Null)
            }

            fn visit_seq<V: SeqAccess<'de>>(self, mut visitor: V) -> Result<Value, V::Error> {
                let mut vec = Vec::new();
                while let Some(element) = visitor.next_element()? {
                    vec.push(element);
                }
                Ok(Value::List(vec))
            }

            fn visit_map<V: MapAccess<'de>>(self, mut visitor: V) -> Result<Value, V::Error> {
                let mut map: HashMap<String, Value> = HashMap::new();
                while let Some((key, value)) = visitor.next_entry()? {
                    map.insert(key, value);
                }
                Ok(Value::Map(map))
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

impl Serialize for Value {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            Value::Null => serializer.serialize_unit(),
            Value::Bool(b) => serializer.serialize_bool(*b),
            Value::Int(n) => n.serialize(serializer),
            Value::String(s) => serializer.serialize_str(s),
            Value::List(seq) => seq.serialize(serializer),
            Value::Map(hash) => {
                use serde::ser::SerializeMap;
                let mut map = serializer.serialize_map(Some(hash.len()))?;
                for (k, v) in hash {
                    map.serialize_entry(k, v)?;
                }
                map.end()
            }
        }
    }
}
