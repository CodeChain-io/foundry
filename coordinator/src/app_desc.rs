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

use std::borrow::Borrow;
use std::collections::{BTreeMap, HashMap};
use std::{fmt, fmt::Display, fmt::Formatter};

use primitives::H256;
use rustc_hex::FromHexError;
use serde::de::{DeserializeOwned, DeserializeSeed, Error, Unexpected};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

use super::values::Value;
use once_cell::sync::Lazy;
use regex::Regex;
use std::fmt::Debug;
use std::ops::{Deref, DerefMut};
use std::str::FromStr;

pub(self) mod params;
pub(self) mod validator;

macro_rules! module_delim {
    () => {
        "/"
    };
}
macro_rules! namespace_delim {
    () => {
        "."
    };
}
macro_rules! first_word {
    () => {
        r"[A-Za-z][a-z0-9]*|[A-Z][A-Z0-9]*"
    };
}
macro_rules! trailing_word {
    () => {
        r"[a-z0-9]+|[A-Z0-9]+"
    };
}
macro_rules! ident {
    () => {
        concat!(first_word!(), "(-", trailing_word!(), ")*")
    };
}
macro_rules! simple_name {
    () => {
        concat!("^", ident!(), "$")
    };
}
macro_rules! local_name {
    () => {
        concat!("^", ident!(), "(", namespace_delim!(), ident!(), ")*$")
    };
}
macro_rules! global_name {
    () => {
        concat!("^", ident!(), module_delim!(), ident!(), "(", namespace_delim!(), ident!(), ")*$")
    };
}
macro_rules! impl_name {
    ($name_type:ident, $pattern:ident, $expecting:tt) => {
        #[derive(Hash, Eq, Ord, PartialOrd, PartialEq)]
        pub struct $name_type(String);

        impl Deref for $name_type {
            type Target = String;

            fn deref(&self) -> &Self::Target {
                &self.0
            }
        }

        impl Borrow<str> for $name_type {
            fn borrow(&self) -> &str {
                &self.0
            }
        }

        impl Debug for $name_type {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                Debug::fmt(&self.0, f)
            }
        }

        impl Display for $name_type {
            fn fmt(&self, f: &mut Formatter) -> fmt::Result {
                Display::fmt(&self.0, f)
            }
        }

        impl<'de> Deserialize<'de> for $name_type {
            fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
                deserializer
                    .deserialize_str(NameVisitor {
                        expecting: $expecting,
                        pattern: &*$pattern,
                    })
                    .map($name_type)
            }
        }
    };
}

pub const MODULE_DELIMITER: &str = module_delim!();
pub const NAMESPACE_DELIMITER: &str = namespace_delim!();

static SIMPLE_NAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(simple_name!()).unwrap());
static LOCAL_NAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(local_name!()).unwrap());
static GLOBAL_NAME_RE: Lazy<Regex> = Lazy::new(|| Regex::new(global_name!()).unwrap());

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct AppDesc {
    // keyed with Name rather than module hash to allow for multiple instances of single module
    pub modules: HashMap<SimpleName, ModuleSetup>,
    /// The ID of the default `Sandboxer` to be used when no `Sandboxer` is specified for modules.
    #[serde(default)]
    pub default_sandboxer: String,
    #[serde(default)]
    pub host: HostSetup,
    #[serde(default)]
    pub transactions: Namespaced<SimpleName>,
    #[serde(default)]
    pub param_defaults: Namespaced<String>,
}

impl AppDesc {
    pub fn from_str(s: &str) -> anyhow::Result<AppDesc> {
        let app_desc: AppDesc = serde_yaml::from_str(s)?;
        app_desc.validate()?;

        Ok(app_desc)
    }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct ModuleSetup {
    #[serde(deserialize_with = "deserialize_h256")]
    pub hash: H256,
    #[serde(default)]
    pub sandboxer: String,
    #[serde(default)]
    pub exports: Namespaced<Constructor>,
    #[serde(default)]
    pub imports: Namespaced<GlobalName>,
    /// List of export names expected to hold the required services.
    /// Then the module will receive imports for `@tx/<transaction-type>/<export-name>`s.
    /// It is mainly intended for modules providing `TxSorter` service.
    #[serde(default)]
    pub transactions: Vec<LocalName>,
    #[serde(default)]
    pub init_config: Value,
    #[serde(default)]
    pub genesis_config: Value,
    #[serde(default)]
    pub tags: HashMap<String, Value>,
}

#[derive(Deserialize, Default, Debug)]
pub struct HostSetup {
    #[serde(default)]
    pub exports: Namespaced<Constructor>,
    #[serde(default)]
    pub imports: Namespaced<GlobalName>,
    #[serde(default)]
    pub init_config: Namespaced<Value>,
    #[serde(default)]
    pub genesis_config: Namespaced<Value>,
}

#[derive(Debug)]
pub struct Constructor {
    pub name: String,
    pub args: Value,
}

struct NameVisitor {
    expecting: &'static str,
    pattern: &'static Regex,
}

impl<'de> Visitor<'de> for NameVisitor {
    type Value = String;

    fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
        write!(formatter, "{}", self.expecting)
    }

    fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
        if !self.pattern.is_match(v) {
            Err(E::invalid_value(Unexpected::Str(v), &self.expecting))
        } else {
            Ok(v.to_owned())
        }
    }
}

impl_name!(SimpleName, SIMPLE_NAME_RE, "a kebab-cased identifier");

impl_name!(LocalName, LOCAL_NAME_RE, "a name consisting of identifiers separated by dots");

impl_name!(GlobalName, GLOBAL_NAME_RE, "a namespaced name qualified with module name");

impl GlobalName {
    pub fn module(&self) -> &str {
        let delimiter_index = self.0.find(MODULE_DELIMITER).expect("a module name followed by a module delimiter");
        &self.0[0..delimiter_index]
    }

    pub fn name(&self) -> &str {
        let delimiter_index = self.0.find(MODULE_DELIMITER).expect("a module name followed by a module delimiter");
        &self.0[delimiter_index + 1..]
    }
}

fn deserialize_h256<'de, D: Deserializer<'de>>(deserializer: D) -> Result<H256, D::Error> {
    struct H256Visitor;

    impl<'de> Visitor<'de> for H256Visitor {
        type Value = H256;

        fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
            write!(formatter, "64 hexadecimals representing a H256")
        }

        fn visit_str<E: Error>(self, v: &str) -> Result<Self::Value, E> {
            let hash = H256::from_str(v).map_err(|e| match e {
                FromHexError::InvalidHexCharacter(_char, _usize) => {
                    let message = &*format!("{:?}", e);
                    E::invalid_value(Unexpected::Str(v), &message)
                }
                FromHexError::InvalidHexLength => E::invalid_length(v.len(), &"64 hex decimals"),
            })?;
            Ok(hash)
        }
    }
    deserializer.deserialize_str(H256Visitor)
}

struct ConstructorVisitor;

impl<'de> Deserialize<'de> for Constructor {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        impl<'de> Visitor<'de> for ConstructorVisitor {
            type Value = Constructor;

            fn expecting(&self, formatter: &mut Formatter) -> fmt::Result {
                formatter.write_str(
                    "a map with single key value pair that serves \
                    as a specification of a constructor call",
                )
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                match map.next_entry()? {
                    Some((name, args)) => match map.next_key::<String>()? {
                        Some(_) => Err(Error::custom("Single constructor must be specified")),
                        None => Ok(Constructor {
                            name,
                            args,
                        }),
                    },
                    None => Err(Error::custom("No constructor specified")),
                }
            }
        }
        deserializer.deserialize_map(ConstructorVisitor)
    }
}

pub struct Namespaced<T: DeserializeOwned>(BTreeMap<String, T>);

impl<T: DeserializeOwned + Debug> Debug for Namespaced<T> {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<T: DeserializeOwned> Default for Namespaced<T> {
    fn default() -> Self {
        Namespaced(Default::default())
    }
}

impl<T: DeserializeOwned> Deref for Namespaced<T> {
    type Target = BTreeMap<String, T>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<T: DeserializeOwned> DerefMut for Namespaced<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

const NAMESPACE_PREFIX: char = '\\';

impl<T: DeserializeOwned> From<Namespaced<T>> for BTreeMap<String, T> {
    fn from(from: Namespaced<T>) -> Self {
        from.0
    }
}

struct NamespacedMapVisitor<'a, T: DeserializeOwned> {
    prefix: String,
    map: &'a mut BTreeMap<String, T>,
}

impl<'a, 'de, T: DeserializeOwned> Visitor<'de> for NamespacedMapVisitor<'a, T> {
    type Value = ();

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a map with a given type or a nested namespace as values")
    }

    fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
        fn to_qualified<'s>(prefix: &'s str, key: &'s str) -> String {
            if prefix.is_empty() {
                key.to_owned()
            } else {
                String::with_capacity(prefix.len() + key.len() + 1) + prefix + NAMESPACE_DELIMITER + key
            }
        }

        while let Some(key) = map.next_key::<String>()? {
            if key.starts_with(NAMESPACE_PREFIX) {
                let key_part = &key[1..];
                if !LOCAL_NAME_RE.is_match(key_part) {
                    return Err(A::Error::invalid_value(Unexpected::Str(&key), &"an @-prefixed qualified name"))
                }
                let prefix = to_qualified(&self.prefix, key_part);
                map.next_value_seed(NamespacedMapVisitor {
                    prefix,
                    map: self.map,
                })?;
            } else {
                if !LOCAL_NAME_RE.is_match(&key) {
                    return Err(A::Error::invalid_value(Unexpected::Str(&key), &"a qualified name"))
                }
                let qualified_key = to_qualified(&self.prefix, &key);
                self.map.insert(qualified_key, map.next_value::<T>()?);
            }
        }
        Ok(())
    }
}

impl<'de, T: DeserializeOwned + 'de> Deserialize<'de> for Namespaced<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let mut map = BTreeMap::new();
        deserializer.deserialize_map(NamespacedMapVisitor {
            prefix: String::new(),
            map: &mut map,
        })?;
        Ok(Namespaced(map))
    }
}

impl<'a, 'de, T: DeserializeOwned> DeserializeSeed<'de> for NamespacedMapVisitor<'a, T> {
    type Value = ();

    fn deserialize<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        deserializer.deserialize_map(self)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::app_desc::AppDesc;
    use unindent::unindent;

    #[test]
    fn load_essentials() {
        let source = unindent(
            r#"
            modules:
                awesome-module:
                    hash: 1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef
                    transactions:
                        - has-seq
                    init-config:
                        test: 1
                        test1:
                            key1: 1
                            key2: sdfsdaf
            host:
                imports:
                    a: awesome-module/a.a
                    \namespace:
                        b.b: asdfsdaf-asdf
            transactions:
                great-tx: awesome-module
            param-defaults:
                num-threads: 10
        "#,
        );
        let _: AppDesc = serde_yaml::from_str(&source).unwrap();
    }
}
