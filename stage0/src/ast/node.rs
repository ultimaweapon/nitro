use super::{Attribute, Class, TypeImpl};
use crate::lexer::Span;

/// An AST node.
pub enum Node {
    Attribute(Attribute),
    Class(Class),
    TypeImpl(TypeImpl),
}

impl Node {
    pub fn span(&self) -> &Span {
        match self {
            Self::Attribute(v) => v.span(),
            Self::Class(v) => v.span(),
            Self::TypeImpl(v) => v.span(),
        }
    }
}

impl From<Attribute> for Node {
    fn from(value: Attribute) -> Self {
        Self::Attribute(value)
    }
}

impl From<Class> for Node {
    fn from(value: Class) -> Self {
        Self::Class(value)
    }
}

impl From<TypeImpl> for Node {
    fn from(value: TypeImpl) -> Self {
        Self::TypeImpl(value)
    }
}
