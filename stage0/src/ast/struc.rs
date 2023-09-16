use super::Attribute;
use crate::lexer::{Identifier, Span, StructKeyword};

/// A struct.
///
/// Struct in Nitro is a value type the same as .NET and its memory layout is always the same as C.
/// All fields must also be a struct and will always public.
///
/// Struct type cannot be a generic type and does not supports inheritance.
pub struct Struct {
    attrs: Vec<Attribute>,
    def: StructKeyword,
    name: Identifier,
}

impl Struct {
    pub fn new(attrs: Vec<Attribute>, def: StructKeyword, name: Identifier) -> Self {
        Self { attrs, def, name }
    }

    pub fn span(&self) -> &Span {
        self.def.span()
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }
}
