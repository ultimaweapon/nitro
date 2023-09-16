use super::Attribute;
use crate::lexer::{ClassKeyword, Identifier, Span};

/// A class.
///
/// Class in Nitro is a reference type, which mean any variable of a class type will be a pointer to
/// the heap allocated. All fields in the class will always private.
pub struct Class {
    attrs: Vec<Attribute>,
    def: ClassKeyword,
    name: Identifier,
}

impl Class {
    pub fn new(attrs: Vec<Attribute>, def: ClassKeyword, name: Identifier) -> Self {
        Self { attrs, def, name }
    }

    pub fn span(&self) -> &Span {
        self.def.span()
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }
}
