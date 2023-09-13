use crate::lexer::{Identifier, ImplKeyword, Span};

/// An implementation block for a type.
pub struct TypeImpl {
    def: ImplKeyword,
    ty: Identifier,
}

impl TypeImpl {
    pub fn new(def: ImplKeyword, ty: Identifier) -> Self {
        Self { def, ty }
    }

    pub fn span(&self) -> &Span {
        self.def.span()
    }
}
