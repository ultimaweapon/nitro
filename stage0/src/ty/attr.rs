/// A collection of attributes.
pub trait Attributes {
    fn public(&self) -> Option<Public>;
    fn repr(&self) -> Option<Representation>;
}

/// Argument of `@pub`.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Public {
    External,
}

/// Argument of `@repr`
#[derive(Clone, Copy)]
pub enum Representation {
    I32,
    U8,
    Un,
}
