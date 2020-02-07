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

use crossbeam_channel::{Receiver, Sender};
use linkme::distributed_slice;
use once_cell::sync;
use std::collections::HashMap;
use std::sync::Arc;
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

/// The list of functions for creating [`Linker`] implementations.
///
/// [`Linker`]: ./trait.Linker.html
#[distributed_slice]
static LINKER_CTORS: [fn() -> Arc<dyn Linker>] = [..];

/// Returns a `Linker` with the given `id`.
pub fn linker(id: &str) -> Option<Arc<dyn Linker>> {
    static MAP: sync::Lazy<HashMap<&'static str, Arc<dyn Linker>>> = sync::Lazy::new(|| {
        LINKER_CTORS
            .iter()
            .map(|new| {
                let linker = new();
                (linker.id(), linker)
            })
            .collect()
    });
    MAP.get(id).map(Arc::clone)
}

/// A linker is responsible for linking to `Port`s if both of them support
/// the required common traits. Each linker must mark itself with `#[Linker]`
/// attribute.
pub trait Linker: Send + Sync {
    /// Returns the identifier for this `Linker`.
    fn id(&self) -> &'static str;

    /// Links the two [`Port`]s together.
    ///
    /// [`Port`]: ./trait.Port.html
    fn link(&self, a: &mut dyn Port, b: &mut dyn Port) -> Result<()>;
}

/// An entity that can be linked with another `Linkable`.
pub trait Linkable: Send + Sync {
    /// Returns a list of [`Linker`] IDs in the order of preference.
    ///
    /// [`Linker`]: ./trait.Linker.html
    fn supported_linkers(&self) -> &'static [&'static str];

    /// Creates a new [`Port`] that can be linked with a [`Linker`].
    ///
    /// [`Port`]: ./trait.Port.html
    /// [`Linker`]: ./trait.Linker.html
    fn new_port(&mut self) -> Arc<dyn Port>;
}

/// A port represents an endpoint of a link between two [`Linkable`]s.
///
/// Before linking two ports, each may be set up with its [`send`] and [`receive`] methods.
/// This trait is just the basic protocol and every `Port` it supposed to implement additional
/// traits for its supported link types.
///
/// [`Linkable`]: ./trait.Linkable.html
/// [`send`]: ./trait.Port.html#tymnethod.send
/// [`receive`]: ./trait.Port.html#tymnethod.receive
pub trait Port: 'static {
    /// Sets to send a list of handles to the other end on link.
    ///
    /// `desc` is encoded in `CBOR`.
    fn send(&mut self, desc: &[u8]) -> &mut dyn Port;

    /// Sets to which slots the handles received from the other end are to be assigned.
    ///
    /// This way, a module can't assign to an arbitrary slot in the other end.
    /// Only to the slots set by the host.
    fn receive(&mut self, slots: &[&str]) -> &mut dyn Port;

    /// Sets the common `Sender` and `Receiver` to link two `BasePort`s.
    ///
    /// This method is to support the base link, which is required for the minimum
    /// interoperability among [`Linkable`]s. Upon a call to this method, `Port` needs
    /// to send and receive handles as configured with [`send`] and [`receive`].
    ///
    /// [`Linkable`]: ./trait.Linkable.html
    /// [`send`]: #tymethod.send
    /// [`receive`]: #tymethod.receive
    fn link(&mut self, sender: Sender<Box<dyn AsRef<[u8]>>>, receiver: Receiver<Box<dyn AsRef<[u8]>>>);
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The port from a linkable '{id}' is not supported by the linker")]
    UnsupportedPortType {
        id: &'static str,
    },
}
