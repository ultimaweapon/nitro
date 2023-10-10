use super::Attributes;
use crate::lexer::Identifier;

/// An implementation of [`crate::ty::BasicType`] for a type from source file.
pub struct BasicType {
    attrs: Attributes,
    is_ref: bool,
    name: Identifier,
}

impl BasicType {
    pub fn new(attrs: Attributes, is_ref: bool, name: Identifier) -> Self {
        Self {
            attrs,
            is_ref,
            name,
        }
    }

    pub fn attrs(&self) -> &Attributes {
        &self.attrs
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }
}

impl crate::ty::BasicType for BasicType {
    fn is_ref(&self) -> bool {
        self.is_ref
    }

    fn attrs(&self) -> &dyn crate::ty::Attributes {
        &self.attrs
    }

    fn name(&self) -> &str {
        self.name.value()
    }
}
