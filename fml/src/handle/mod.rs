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

pub mod call;
pub mod dispatch;
pub mod id;
pub mod table;

use super::port::PortId;
pub use dispatch::PortDispatcher;
use serde::{Deserialize, Serialize};

pub type MethodId = u32;
pub type TraitId = u16;
pub type InstanceId = u16;

pub const ID_ORDERING: std::sync::atomic::Ordering = std::sync::atomic::Ordering::Relaxed;
pub type MethodIdAtomic = std::sync::atomic::AtomicU32;
pub type TraitIdAtomic = std::sync::atomic::AtomicU16;

// We avoid using additional space with Option<>, by these.
pub const UNDECIDED_INDEX: InstanceId = std::u16::MAX;
pub const UNDECIDED_TRAIT: TraitId = std::u16::MAX;
pub const UNDECIDED_PORT: PortId = std::u16::MAX;

/// This struct represents an index to a service object in port server's registry
#[derive(PartialEq, Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ServiceObjectId {
    /// This is for debug. It is not used in call / dispatch.
    pub(crate) trait_id: TraitId,
    pub(crate) index: InstanceId,
}

/// This struct is stored in both service object and call stub.
/// Actually, both use only part of the fields respectively,
/// though the other fields will be still set same as
/// the other side's HandleInstance.
/// For debugging and simplicity, we won't split this for now.
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct HandleInstance {
    pub(crate) id: ServiceObjectId,
    // That of exporter's.
    pub(crate) port_id_exporter: PortId,
    // That of importer's.
    pub(crate) port_id_importer: PortId,
}

impl Default for HandleInstance {
    fn default() -> Self {
        HandleInstance {
            id: ServiceObjectId {
                trait_id: UNDECIDED_TRAIT,
                index: UNDECIDED_INDEX,
            },
            port_id_exporter: UNDECIDED_PORT,
            port_id_importer: UNDECIDED_PORT,
        }
    }
}

impl HandleInstance {
    /// This clone is allowed only within this crate
    pub(crate) fn careful_clone(&self) -> Self {
        HandleInstance {
            id: self.id,
            port_id_exporter: self.port_id_exporter,
            port_id_importer: self.port_id_importer,
        }
    }

    /// You (module implementor) should not call this!
    pub fn for_dispatcher_get_port_id(&self) -> PortId {
        self.port_id_exporter
    }
}

/// All service trait must has this as a supertrait.
pub trait Service: dispatch::ServiceDispatcher + std::fmt::Debug + intertrait::CastFrom + Send + Sync {
    fn get_handle(&self) -> &HandleInstance;
    fn get_handle_mut(&mut self) -> &mut HandleInstance;
}

/// TODO: Replace this with LinkBootstrapping.
#[derive(PartialEq, Serialize, Deserialize, Debug)]
pub struct HandleExchange {
    /// Id of exporter (same as that in Config)
    pub exporter: String,
    /// Id of importer (same as that in Config)
    pub importer: String,
    /// Handles. Importer must cast these to Box<dyn SomeHandle> itself.
    pub handles: Vec<HandleInstance>,
    /// Opaque argument
    pub argument: Vec<u8>,
}

/// TODO: Replace this with LinkBootstrapping.
/// We assume that there could be at most one link for a pair of modules in this exchange phase,
/// so no information about PortId is carried.
pub trait HandlePreset {
    fn export() -> Vec<HandleExchange>;
    fn import(exchange: HandleExchange);
}

/// These provides compile time resolution of associated type/function for a given handle trait.
pub mod association {
    use super::*;

    pub trait Dispatch {
        type T: ?Sized;
        fn dispatch(object: &Self::T, method: MethodId, arguments: &[u8], return_buffer: std::io::Cursor<&mut Vec<u8>>);
    }

    pub trait Import {
        type T: ?Sized;
        fn import(handle: HandleInstance) -> Box<Self::T>;
    }

    pub trait Export {
        type T: ?Sized;
        fn export(port_id: PortId, handle: Box<Self::T>) -> HandleInstance;
    }
}
