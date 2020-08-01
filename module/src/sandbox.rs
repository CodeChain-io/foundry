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

use crate::link::Linkable;
use linkme::distributed_slice;
use once_cell::sync;
use std::collections::HashMap;
use std::fmt::Debug;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use thiserror::Error;

#[distributed_slice]
pub static SANDBOXERS: [fn() -> Arc<dyn Sandboxer>] = [..];

/// Returns a `Sandboxer` with the given `id`.
pub fn sandboxer(id: &str) -> Option<Arc<dyn Sandboxer>> {
    static MAP: sync::Lazy<HashMap<&'static str, Arc<dyn Sandboxer>>> = sync::Lazy::new(|| {
        SANDBOXERS
            .iter()
            .map(|new| {
                let sandboxer = new();
                (sandboxer.id(), sandboxer)
            })
            .collect()
    });
    MAP.get(id).map(Arc::clone)
}

/// An entity that can sandbox modules of types it supports.
///
/// A `Sandboxer` is thread-safe.
pub trait Sandboxer: Send + Sync {
    /// Returns the identifier string for this provider.
    fn id(&self) -> &'static str;

    /// Returns a list of module types that can be loaded by this `Sandboxer`.
    fn supported_module_types(&self) -> &'static [&'static str];

    /// Loads the module in the given `path` into a [`Sandbox`] and pass the given
    /// `init` and `exports` to the module for initialization.
    ///
    /// The corresponding module must have been imported into the module repository
    /// configured for the current Foundry host. That is why this method accepts a `path`
    /// to identify a module.
    ///
    /// The `init` serves as configuration parameters for the module-wide initialization.
    /// It must be a trait object for `erased_serde::Serialize` to allow for serialization
    /// into whatever format the receiver likes.
    ///
    /// And the `exports` instruct how to instantiate an ordered list of service objects
    /// to be exported via links. Each item in the `exports` designates a call on a module's
    /// constructor service, where the first element is name of a constructor function,
    /// and the second element is arguments to the constructor function, which is a trait
    /// object for `erased_serde::Serialize` to allow for serialization into whatever format
    /// the receiver likes.
    ///
    /// [`Sandbox`]: ./trait.Sandbox.html
    fn load(
        &self,
        path: &dyn AsRef<Path>,
        init: &dyn erased_serde::Serialize,
        exports: &[(&str, &dyn erased_serde::Serialize)],
    ) -> Result<Box<dyn Sandbox>, LoadError>;
}

/// A sandbox instance hosting an instantiated module.
pub trait Sandbox: Linkable {
    fn debug(&mut self, _arg: &[u8]) -> Vec<u8> {
        Vec::new()
    }
}

#[derive(Debug, Error)]
pub enum LoadError {
    /// The module in the given `path` is suspected as corrupted.
    #[error("The module at '{path:?}'seems corrupted")]
    ModuleCorrupted {
        /// The path to the subject module.
        path: PathBuf,
        source: Option<anyhow::Error>,
    },

    /// An error specific to the `Sandboxer` involved.
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}
