use super::Attributes;
use crate::lexer::{Identifier, Span, StructKeyword};

/// An implementation of [`crate::ty::Struct`].
pub struct Struct {
    attrs: Attributes,
    def: StructKeyword,
    name: Identifier,
}

impl Struct {
    pub fn new(attrs: Attributes, def: StructKeyword, name: Identifier) -> Self {
        Self { attrs, def, name }
    }

    pub fn span(&self) -> &Span {
        self.def.span()
    }

    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }
}

impl crate::ty::Struct for Struct {
    fn attrs(&self) -> &dyn crate::ty::Attributes {
        &self.attrs
    }

    fn name(&self) -> &str {
        self.name.value()
    }
}
