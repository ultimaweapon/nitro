use super::{Attribute, Statement, Type};
use crate::lexer::{FnKeyword, Identifier};

/// A function.
pub struct Function {
    attrs: Vec<Attribute>,
    def: FnKeyword,
    name: Identifier,
    params: Vec<FunctionParam>,
    ret: Option<Type>,
    body: Option<Vec<Statement>>,
}

impl Function {
    pub fn new(
        attrs: Vec<Attribute>,
        def: FnKeyword,
        name: Identifier,
        params: Vec<FunctionParam>,
        ret: Option<Type>,
        body: Option<Vec<Statement>>,
    ) -> Self {
        Self {
            attrs,
            def,
            name,
            params,
            ret,
            body,
        }
    }
}

/// A function parameter.
pub struct FunctionParam {
    name: Identifier,
    ty: Type,
}

impl FunctionParam {
    pub fn new(name: Identifier, ty: Type) -> Self {
        Self { name, ty }
    }
}
