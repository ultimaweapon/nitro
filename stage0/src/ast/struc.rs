use super::{Attributes, Representation};
use crate::lexer::{Identifier, Span, StructKeyword};

/// A struct.
///
/// Struct in Nitro is a value type the same as .NET and its memory layout is always the same as C.
/// All fields must also be a struct and will always public.
///
/// Struct type cannot be a generic type and does not supports inheritance.
pub enum Struct {
    Primitive(Attributes, Representation, StructKeyword, Identifier),
    Composite(Attributes, StructKeyword, Identifier),
}

impl Struct {
    pub fn span(&self) -> &Span {
        let def = match self {
            Self::Primitive(_, _, d, _) => d,
            Self::Composite(_, d, _) => d,
        };

        def.span()
    }

    pub fn attrs(&self) -> &Attributes {
        match self {
            Self::Primitive(a, _, _, _) => a,
            Self::Composite(a, _, _) => a,
        }
    }

    pub fn name(&self) -> &Identifier {
        match self {
            Self::Primitive(_, _, _, i) => i,
            Self::Composite(_, _, i) => i,
        }
    }
}
