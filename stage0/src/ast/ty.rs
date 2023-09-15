use crate::codegen::{Codegen, LlvmType, LlvmVoid};
use crate::lexer::{Asterisk, CloseParenthesis, ExclamationMark, Identifier, OpenParenthesis};

/// A type of something (e.g. variable).
pub struct Type {
    prefixes: Vec<Asterisk>,
    name: TypeName,
}

impl Type {
    pub fn new(prefixes: Vec<Asterisk>, name: TypeName) -> Self {
        Self { prefixes, name }
    }

    pub fn build<'a>(&self, cx: &'a Codegen) -> Option<LlvmType<'a>> {
        let mut ty = match &self.name {
            TypeName::Unit(_, _) => Some(LlvmType::Void(LlvmVoid::new(cx))),
            TypeName::Never(_) => None,
            TypeName::Ident(_) => todo!(),
        };

        ty
    }
}

pub enum TypeName {
    Unit(OpenParenthesis, CloseParenthesis),
    Never(ExclamationMark),
    Ident(Vec<Identifier>),
}
