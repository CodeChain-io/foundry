use std::fmt::Debug;
use std::sync::Arc;

use linkme::distributed_slice;
use primitives::H256;
use thiserror::Error;

use crate::link::Linkable;

type Result<T> = std::result::Result<T, Error>;

#[distributed_slice]
pub static SANDBOXERS: [fn() -> Arc<dyn Sandboxer>] = [..];

/// An entity that can sandbox modules of types it supports.
///
/// A `Sandboxer` is thread-safe.
pub trait Sandboxer {
    /// Returns the identifier string for this provider.
    fn id(&self) -> &'static str;

    /// Returns a list of module types that can be loaded by this `Sandboxer`.
    fn supported_module_types(&self) -> &'static [&'static str];

    /// Loads the module identified by the given `hash` into a [`Sandbox`].
    ///
    /// The corresponding module must have been imported into the module repository
    /// configured for the current Foundry host.
    ///
    /// [`Sandbox`]: ./trait.Sandbox.html
    fn load(&self, hash: &dyn AsRef<H256>) -> Result<Arc<dyn Sandbox>>;
}

/// A sandbox instance hosting an instantiated module.
pub trait Sandbox: Linkable {
    /// Returns the `Sandboxer` for this sandbox.
    fn sandboxer(&self) -> Arc<dyn Sandboxer>;
}

#[derive(Debug, Error)]
pub enum Error {
    /// The module identified by the given `H256` is not in the module repository.
    #[error("Could not find the specified module: {id:?}")]
    ModuleNotFound {
        id: H256,
    },

    /// The module identified by the given `H256` is not supported by the provider.
    #[error("The module is not supported: type '{ty:?}', id '{id:?}'")]
    UnsupportedModuleType {
        /// The identifier of the subject module.
        id: H256,
        /// The type of the subject module module.
        ty: String,
    },
}
