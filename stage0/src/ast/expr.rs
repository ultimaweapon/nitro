use super::Statement;
use crate::lexer::{
    AsmKeyword, Equals, ExclamationMark, Identifier, IfKeyword, NullKeyword, StringLiteral,
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

/// A function call.
pub struct Call {
    path: Vec<Identifier>,
    name: Identifier,
    args: Vec<Vec<Expression>>,
}

impl Call {
    pub fn new(path: Vec<Identifier>, name: Identifier, args: Vec<Vec<Expression>>) -> Self {
        Self { path, name, args }
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
}
