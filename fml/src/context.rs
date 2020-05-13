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

use crate::port::Port;
use crate::port::PortId;
use once_cell::sync::OnceCell;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Config {
    /// kind of this module. Per-binary
    pub kind: String,
    /// id of this instance of module. Per-instance, Per-appdescriptor
    pub id: String,
    /// key of this instance of module. Per-instance, Per-execution, Per-node
    pub key: single_process_support::InstanceKey,
    /// Arguments given to this module.
    pub args: Vec<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FmlConfig {
    /// Number of inbound call handlers
    pub server_threads: usize,
    /// Maximum outbound call slots
    pub call_slots: usize,
}

/// You can add additional variables as you want.
pub trait Custom {
    fn new(context: &Config) -> Self;
}

/// Though it uses HashMap now, we can issue PortIds in a series from 0 to ...
/// Thus it may be optimized out to use plain array later.
pub struct PortTable {
    /// (Counterparty module's config, Counterarty's port id, Actual port)
    pub map: HashMap<PortId, (Config, PortId, Port)>,
    /// If this is true, the host is trying to shutdown all the modules
    /// You won't request deletion of handle because it doesn't matter.
    pub no_drop: bool,
}

impl PortTable {
    // TODO: Remove this in favor of the LinkBootstrapping
    /// Find first occurence of given moudle Id and return corresponding PortId
    pub fn find(&self, id: &str) -> Result<PortId, ()> {
        Ok(*self.map.iter().find(|&(_, (config, ..))| config.id == id).ok_or(())?.0)
    }
}

/// A global context that will be accessible from this module
pub struct Context<T: Custom> {
    /// Module author should not care about this.
    pub ports: Arc<RwLock<PortTable>>,

    /// Meta, pre-decided constant variables
    pub config: Config,

    /// FML configurations
    pub config_fml: FmlConfig,

    /// Custom variables
    pub custom: T,
}

impl<T: Custom> Context<T> {
    pub fn new(ports: Arc<RwLock<PortTable>>, config: Config, config_fml: FmlConfig, custom: T) -> Self {
        Context {
            ports,
            config,
            config_fml,
            custom,
        }
    }
}

/// This manages thread-local keys for module instance discrimination
/// in the intra-process setup.
/// This instance key setup will happen always but
/// costly global context resolution will be optionally compiled only with the
/// --features "single_process".
/// Note that you must manually set this key before invoke any call if you created
/// threads during service handling
pub mod single_process_support {
    pub type InstanceKey = u32;
    use std::cell::Cell;
    thread_local!(static INSTANCE_KEY: Cell<InstanceKey> = Cell::new(0));

    pub fn set_key(key: InstanceKey) {
        INSTANCE_KEY.with(|k| {
            assert_eq!(k.get(), 0);
            k.set(key);
        })
    }

    pub fn get_key() -> InstanceKey {
        INSTANCE_KEY.with(|k| {
            assert_ne!(k.get(), 0);
            k.get()
        })
    }
}
pub use single_process_support::InstanceKey;
pub const INSTANCE_KEY_MAX: usize = 10000;

// These modules 'global' are accessed from call/dispatch.
// User code won't ever be aware of this.

#[cfg(feature = "single_process")]
pub mod global {
    use super::*;

    static POOL: OnceCell<RwLock<HashMap<InstanceKey, Arc<RwLock<PortTable>>>>> = OnceCell::new();

    fn get_pool_raw() -> &'static RwLock<HashMap<InstanceKey, Arc<RwLock<PortTable>>>> {
        POOL.get_or_init(|| RwLock::new(HashMap::new()))
    }

    pub fn get() -> Arc<RwLock<PortTable>> {
        get_pool_raw()
            .read()
            .unwrap()
            .get(&single_process_support::get_key())
            .expect("Global context is not set.")
            .clone()
    }

    pub fn set(port_table: Arc<RwLock<PortTable>>) {
        assert!(
            get_pool_raw().write().unwrap().insert(single_process_support::get_key(), port_table).is_none(),
            "Global context has been already set"
        );
    }

    pub fn remove() {
        get_pool_raw().write().unwrap().remove(&single_process_support::get_key()).unwrap();
    }
}

#[cfg(not(feature = "single_process"))]
pub mod global {
    use super::*;

    static POOL: OnceCell<Option<Arc<RwLock<PortTable>>>> = OnceCell::new();

    pub fn get() -> Arc<RwLock<PortTable>> {
        POOL.get_or_init(|| None).as_ref().expect("Global context is not set.").clone()
    }

    pub fn set(port_table: Arc<RwLock<PortTable>>) {
        POOL.set(Some(port_table)).map_err(|_| "Global context has been already set").unwrap()
    }

    pub fn remove() {
        // we're shutting the entire program. No need of a manual drop!
    }
}
