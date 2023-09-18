use super::{Path, Statement};
use crate::lexer::{
    AsmKeyword, Equals, ExclamationMark, Identifier, IfKeyword, NullKeyword, Span, StringLiteral,
    UnsignedLiteral,
};

/// An expression.
pub enum Expression {
    Value(Identifier),
    Call(Call),
    Equal(Equals, Equals),
    NotEqual(ExclamationMark, Equals),
    Unsigned(UnsignedLiteral),
    String(StringLiteral),
    Null(NullKeyword),
    Asm(Asm),
    If(If),
}

impl Expression {
    pub fn span(&self) -> Span {
        match self {
            Self::Value(v) => v.span().clone(),
            Self::Call(v) => v.span(),
            Self::Equal(f, s) => f.span() + s.span(),
            Self::NotEqual(f, s) => f.span() + s.span(),
            Self::Unsigned(v) => v.span().clone(),
            Self::String(v) => v.span().clone(),
            Self::Null(v) => v.span().clone(),
            Self::Asm(v) => v.span().clone(),
            Self::If(v) => v.span().clone(),
        }
    }
}

/// A function call.
pub struct Call {
    name: Path,
    args: Vec<Vec<Expression>>,
}

impl Call {
    pub fn new(name: Path, args: Vec<Vec<Expression>>) -> Self {
        Self { name, args }
    }

    pub fn span(&self) -> Span {
        self.name.span()
    }
}

/// An inline assembly (e.g. `asm("nop")`).
pub struct Asm {
    def: AsmKeyword,
    inst: StringLiteral,
    inputs: Vec<(AsmIn, Vec<Expression>)>,
    outputs: Vec<(AsmOut, Identifier)>,
}

impl Asm {
    pub fn new(
        def: AsmKeyword,
        inst: StringLiteral,
        inputs: Vec<(AsmIn, Vec<Expression>)>,
        outputs: Vec<(AsmOut, Identifier)>,
    ) -> Self {
        Self {
            def,
            inst,
            inputs,
            outputs,
        }
    }

    pub fn span(&self) -> &Span {
        self.def.span()
    }
}

/// An input of the inline assembly (e.g. `in("rax")`).
pub enum AsmIn {
    Register(StringLiteral),
}

/// An output of the inline assembly (e.h. `out("rax")`).
pub enum AsmOut {
    Never(ExclamationMark),
}

/// An if expression.
pub struct If {
    def: IfKeyword,
    cond: Vec<Expression>,
    body: Vec<Statement>,
}

impl If {
    pub fn new(def: IfKeyword, cond: Vec<Expression>, body: Vec<Statement>) -> Self {
        Self { def, cond, body }
    }

    pub fn span(&self) -> &Span {
        self.def.span()
    }
}
