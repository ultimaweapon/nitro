use super::Function;
use crate::lexer::{Identifier, ImplKeyword, Span};

/// An implementation block for a type.
pub struct TypeImpl {
    def: ImplKeyword,
    ty: Identifier,
    functions: Vec<Function>,
}

impl TypeImpl {
    pub fn new(def: ImplKeyword, ty: Identifier, functions: Vec<Function>) -> Self {
        Self { def, ty, functions }
    }

    pub fn span(&self) -> &Span {
        self.def.span()
    }

    pub fn functions(&self) -> &[Function] {
        self.functions.as_ref()
    }
}
