use super::Expression;
use crate::lexer::{AttributeName, Span};

/// An attribute.
pub struct Attribute {
    name: AttributeName,
    args: Option<Vec<Vec<Expression>>>,
}

impl Attribute {
    pub fn new(name: AttributeName, args: Option<Vec<Vec<Expression>>>) -> Self {
        Self { name, args }
    }

    pub fn span(&self) -> &Span {
        self.name.span()
    }
}
