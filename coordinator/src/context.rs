/// A `Context` provides the interface against the system services such as
pub trait Context {}

#[derive(Default)]
pub struct DummyContext {}

impl Context for DummyContext {}
