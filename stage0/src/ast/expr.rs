use super::{Path, Statement};
use crate::lexer::{
    AsmKeyword, Equals, ExclamationMark, Identifier, IfKeyword, Lexer, NullKeyword, Span,
    StringLiteral, SyntaxError, Token, UnsignedLiteral,
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

    pub fn parse_args(lex: &mut Lexer) -> Result<Vec<Vec<Self>>, SyntaxError> {
        let mut args = Vec::new();

        loop {
            // Check for ')'.
            match lex.next()? {
                Some(Token::CloseParenthesis(_)) => break,
                Some(_) => lex.undo(),
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect ')' after this",
                    ));
                }
            }

            // Parse expression.
            args.push(Self::parse(lex)?);

            // Check for ','.
            match lex.next()? {
                Some(Token::Comma(_)) => {}
                Some(Token::CloseParenthesis(_)) => break,
                Some(v) => return Err(SyntaxError::new(v.span().clone(), "expect ')'")),
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect ')' after this",
                    ));
                }
            }
        }

        Ok(args)
    }

    pub fn parse(lex: &mut Lexer) -> Result<Vec<Self>, SyntaxError> {
        let mut exprs = Vec::new();

        loop {
            // Check the first item.
            let next = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap().clone(),
                        "expect an expression after this",
                    ));
                }
            };

            let ident = match next {
                Token::Identifier(v) => v,
                Token::UnsignedLiteral(v) => {
                    exprs.push(Expression::Unsigned(v));
                    continue;
                }
                Token::StringLiteral(v) => {
                    exprs.push(Expression::String(v));
                    continue;
                }
                Token::NullKeyword(v) => {
                    exprs.push(Expression::Null(v));
                    break;
                }
                Token::AsmKeyword(v) => {
                    exprs.push(Expression::Asm(Self::parse_asm(lex, v)?));
                    continue;
                }
                Token::IfKeyword(v) => {
                    exprs.push(Expression::If(Self::parse_if(lex, v)?));
                    continue;
                }
                _ => {
                    lex.undo();
                    break;
                }
            };

            // Check the token after the identifier.
            let second = match lex.next()? {
                Some(v) => v,
                None => {
                    exprs.push(Expression::Value(ident));
                    break;
                }
            };

            match second {
                Token::ExclamationMark(ex) => {
                    let eq = lex.next_equals()?;

                    exprs.push(Expression::Value(ident));
                    exprs.push(Expression::NotEqual(ex, eq));
                    continue;
                }
                Token::Equals(eq1) => {
                    let eq2 = lex.next_equals()?;

                    exprs.push(Expression::Value(ident));
                    exprs.push(Expression::Equal(eq1, eq2));
                    continue;
                }
                Token::OpenParenthesis(_) => {
                    let args = Self::parse_args(lex)?;
                    let name = Path::new(vec![Token::Identifier(ident)]);

                    exprs.push(Expression::Call(Call::new(name, args)));
                    continue;
                }
                _ => {
                    lex.undo();
                    exprs.push(Expression::Value(ident));
                    break;
                }
            }
        }

        Ok(exprs)
    }

    fn parse_if(lex: &mut Lexer, def: IfKeyword) -> Result<If, SyntaxError> {
        // Parse condition.
        let exprs = Self::parse(lex)?;
        lex.next_oc()?;

        // Parse the body.
        let body = Statement::parse_block(lex)?;

        Ok(If::new(def, exprs, body))
    }

    fn parse_asm(lex: &mut Lexer, def: AsmKeyword) -> Result<Asm, SyntaxError> {
        lex.next_op()?;

        // Get the instruction.
        let next = match lex.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an instruction after this",
                ));
            }
        };

        let inst = match next {
            Token::StringLiteral(v) => v,
            t => return Err(SyntaxError::new(t.span().clone(), "expect an instruction")),
        };

        // Parse the arguments.
        let mut inputs = Vec::new();
        let mut outputs = Vec::new();

        match lex.next()? {
            Some(Token::Comma(_)) => loop {
                let next = match lex.next()? {
                    Some(v) => v,
                    None => {
                        return Err(SyntaxError::new(
                            lex.last().unwrap().clone(),
                            "expect ')' after this",
                        ));
                    }
                };

                match next {
                    Token::Identifier(v) => {
                        match v.value() {
                            "in" => inputs.push(Self::parse_asm_in(lex)?),
                            "out" => outputs.push(Self::parse_asm_out(lex)?),
                            _ => {
                                return Err(SyntaxError::new(v.span().clone(), "unknown argument"));
                            }
                        }

                        // Check for comma.
                        let next = match lex.next()? {
                            Some(v) => v,
                            None => {
                                return Err(SyntaxError::new(
                                    lex.last().unwrap().clone(),
                                    "expect ')' after this",
                                ));
                            }
                        };

                        match next {
                            Token::Comma(_) => {}
                            Token::CloseParenthesis(_) => break,
                            t => return Err(SyntaxError::new(t.span().clone(), "expect ')'")),
                        }
                    }
                    Token::CloseParenthesis(_) => break,
                    t => return Err(SyntaxError::new(t.span().clone(), "expect ')'")),
                }
            },
            Some(Token::CloseParenthesis(_)) => {}
            Some(t) => return Err(SyntaxError::new(t.span().clone(), "expect ')'")),
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect ')' after this",
                ));
            }
        }

        Ok(Asm::new(def, inst, inputs, outputs))
    }

    fn parse_asm_in(lex: &mut Lexer) -> Result<(AsmIn, Vec<Self>), SyntaxError> {
        // Load target register.
        lex.next_op()?;

        let reg = match lex.next()? {
            Some(v) => match v {
                Token::StringLiteral(v) => AsmIn::Register(v),
                t => return Err(SyntaxError::new(t.span().clone(), "invalid input")),
            },
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an item after this",
                ));
            }
        };

        // Load the value.
        lex.next_cp()?;

        Ok((reg, Self::parse(lex)?))
    }

    fn parse_asm_out(lex: &mut Lexer) -> Result<(AsmOut, Identifier), SyntaxError> {
        // Load output register.
        lex.next_op()?;

        let reg = match lex.next()? {
            Some(v) => match v {
                Token::ExclamationMark(v) => AsmOut::Never(v),
                t => return Err(SyntaxError::new(t.span().clone(), "invalid output")),
            },
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an item after this",
                ));
            }
        };

        // Load the target variable.
        let cp = lex.next_cp()?;
        let var = match lex.next()? {
            Some(Token::Identifier(v)) => v,
            Some(t) => return Err(SyntaxError::new(t.span().clone(), "expect an identifier")),
            None => {
                return Err(SyntaxError::new(
                    cp.span().clone(),
                    "expect an identifier after this",
                ));
            }
        };

        Ok((reg, var))
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
