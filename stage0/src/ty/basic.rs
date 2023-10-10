use super::Attributes;

/// A struct or class.
///
/// Struct in Nitro is a value type the same as .NET and its memory layout is always the same as C.
/// All fields must be a struct and it will always public. Struct type cannot be a generic type and
/// does not supports inheritance.
///
/// Class in Nitro is a reference type, which mean any variable of a class type will be a pointer to
/// the heap allocated. All fields in the class will always private.
pub trait BasicType {
    fn is_ref(&self) -> bool;
    fn attrs(&self) -> &dyn Attributes;
    fn name(&self) -> &str;
}
