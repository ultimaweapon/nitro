use super::{Attribute, Representation};
use crate::lexer::{Identifier, Span, StructKeyword};

/// A struct.
///
/// Struct in Nitro is a value type the same as .NET and its memory layout is always the same as C.
/// All fields must also be a struct and will always public.
///
/// Struct type cannot be a generic type and does not supports inheritance.
pub enum Struct {
    Primitive(Vec<Attribute>, Representation, StructKeyword, Identifier),
    Composite(Vec<Attribute>, StructKeyword, Identifier),
}

impl Struct {
    pub fn span(&self) -> &Span {
        let def = match self {
            Self::Primitive(_, _, d, _) => d,
            Self::Composite(_, d, _) => d,
        };

        def.span()
    }

    pub fn name(&self) -> &Identifier {
        match self {
            Self::Primitive(_, _, _, i) => i,
            Self::Composite(_, _, i) => i,
        }
    }
}
