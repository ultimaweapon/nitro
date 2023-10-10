use super::Attributes;

/// A struct.
///
/// Struct in Nitro is a value type the same as .NET and its memory layout is always the same as C.
/// All fields must also be a struct and will always public.
///
/// Struct type cannot be a generic type and does not supports inheritance.
pub trait Struct {
    fn attrs(&self) -> &dyn Attributes;
    fn name(&self) -> &str;
}
