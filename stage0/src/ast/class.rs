use super::Attributes;
use crate::lexer::{ClassKeyword, Identifier};

/// A class.
///
/// Class in Nitro is a reference type, which mean any variable of a class type will be a pointer to
/// the heap allocated. All fields in the class will always private.
pub struct Class {
    attrs: Attributes,
    def: ClassKeyword,
    name: Identifier,
}

impl Class {
    pub fn new(attrs: Attributes, def: ClassKeyword, name: Identifier) -> Self {
        Self { attrs, def, name }
    }

    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }
}
