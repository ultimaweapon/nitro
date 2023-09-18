use super::Expression;
use crate::lexer::{AttributeName, Span};

/// An attribute.
pub enum Attribute {
    Pub(AttributeName),
    Cfg(AttributeName, Vec<Expression>),
    Ext(AttributeName, Extern),
    Repr(AttributeName, Representation),
    Custom(AttributeName, Option<Vec<Vec<Expression>>>),
}

impl Attribute {
    pub fn span(&self) -> &Span {
        match self {
            Attribute::Pub(n) => n.span(),
            Attribute::Cfg(n, _) => n.span(),
            Attribute::Ext(n, _) => n.span(),
            Attribute::Repr(n, _) => n.span(),
            Attribute::Custom(n, _) => n.span(),
        }
    }
}

/// Argument of `@ext`.
pub enum Extern {
    C,
}

/// Argument of `@repr`
pub enum Representation {
    U8,
    Un,
}
