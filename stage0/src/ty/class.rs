use super::Attributes;

/// A class.
///
/// Class in Nitro is a reference type, which mean any variable of a class type will be a pointer to
/// the heap allocated. All fields in the class will always private.
pub trait Class {
    fn attrs(&self) -> &dyn Attributes;
    fn name(&self) -> &str;
}
