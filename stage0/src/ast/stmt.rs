use super::{Attribute, Expression};
use crate::lexer::{Identifier, LetKeyword};

/// A statement.
pub enum Statement {
    Let(Let),
    Unit(Vec<Expression>),
    Value(Vec<Expression>),
}

/// A let statement.
pub struct Let {
    attrs: Vec<Attribute>,
    def: LetKeyword,
    var: Identifier,
    val: Vec<Expression>,
}

impl Let {
    pub fn new(
        attrs: Vec<Attribute>,
        def: LetKeyword,
        var: Identifier,
        val: Vec<Expression>,
    ) -> Self {
        Self {
            attrs,
            def,
            var,
            val,
        }
    }
}
