use super::Attributes;
use crate::lexer::Identifier;

/// A struct or class in a source file.
pub(super) struct BasicType {
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

    pub fn is_ref(&self) -> bool {
        self.is_ref
    }

    pub fn name(&self) -> &Identifier {
        &self.name
    }
}
