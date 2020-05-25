use std::collections::HashMap;
use std::sync::Arc;

use intertrait::CastFromSync;
use linkme::distributed_slice;
use once_cell::sync;
use thiserror::Error;

type Result<T> = std::result::Result<T, Error>;

/// The list of functions for creating [`Linker`] implementations.
///
/// [`Linker`]: ./trait.Linker.html
#[distributed_slice]
pub static LINKERS: [fn() -> Arc<dyn Linker>] = [..];

/// Returns a `Linker` with the given `id`.
pub fn linker(id: &str) -> Option<Arc<dyn Linker>> {
    static MAP: sync::Lazy<HashMap<&'static str, Arc<dyn Linker>>> = sync::Lazy::new(|| {
        LINKERS
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
/// Before linking two ports, each may be set up with its [`export`] and [`import`] methods.
/// This trait is just the basic protocol and every `Port` it supposed to implement additional
/// traits for its supported link types.
///
/// [`Linkable`]: ./trait.Linkable.html
/// [`export`]: ./trait.Port.html#tymnethod.export
/// [`import`]: ./trait.Port.html#tymnethod.import
pub trait Port: CastFromSync {
    /// Sets to send a list of handles represented by the `ids` to the other end on link
    /// creation. The `ids` are indices into a list of service objects created when the module
    /// owning this port is loaded into a sandbox.CBOR map fed
    /// to the constructor function.
    fn export(&mut self, ids: &[usize]) -> &mut dyn Port;

    /// Sets to which slots the handles received from the other end are to be assigned.
    ///
    /// This way, a module can't assign to an arbitrary slot in the other end.
    /// Only to the slots set by the host.
    fn import(&mut self, slots: &[&str]) -> &mut dyn Port;

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
pub trait Receiver: Send {
    /// Places the given message (`[u8]`) and returns immediately.
    /// The `message` is typed `Box<dyn AsRef<[u8]>>` to allow for zero copy sending
    /// as much as possible. The intention is to wrap various types as they are if they
    /// can be converted into a `&[u8]` to pass into this method. The `Box` is dropped
    /// when done with the data, and the underlying data will also be dropped then.
    fn receive(&mut self, message: Box<dyn AsRef<[u8]>>);
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("The port from a linkable '{id}' is not supported by the linker")]
    UnsupportedPortType {
        id: &'static str,
    },
}
