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

use std::sync::Arc;

use super::Port;

pub trait BasePort: Port {
    /// Returns the [`Receiver`] for placing messages into the [`Linkable`] this `Port` is
    /// created from. This method is used to implement the base link, which is required
    /// for the minimum interoperability among [`Linkable`]s.
    ///
    /// [`Receiver`]: ./trait.Receiver.html
    /// [`Linkable`]: ./trait.Linkable.html
    fn receiver(&self) -> Arc<dyn Receiver>;

    /// Links with another [`Linkable`] by passing in a [`Receiver`] taken from the [`Linkable`]
    /// in the opposite side.
    ///
    /// This method is to support the base link, which is required for the minimum
    /// interoperability among [`Linkable`]s. Upon a call to this method,
    /// `Port`s need to send and receive handles as configured with [`export`] and [`import`].
    ///
    /// [`Linkable`]: ./trait.Linkable.html
    /// [`export`]: #tymethod.export
    /// [`import`]: #tymethod.import
    fn link(&mut self, receiver: Arc<dyn Receiver>);
}

/// An endpoint implemented by a [`Linkable`] for receiving incoming calls
/// from another [`Linkable`].
///
/// [`Linkable`]: ./trait.Linkable.html
pub trait Receiver: Send + Sync {
    /// Places the given message (`[u8]`) and returns immediately.
    /// The `message` is typed `Box<dyn AsRef<[u8]>>` to allow for zero copy sending
    /// as much as possible. The intention is to wrap various types as they are if they
    /// can be converted into a `&[u8]` to pass into this method. The `Box` is dropped
    /// when done with the data, and the underlying data will also be dropped then.
    fn receive(&mut self, message: Box<dyn AsRef<[u8]>>);
}
