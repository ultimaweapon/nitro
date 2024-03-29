use super::expr::Expression;
use crate::codegen::Codegen;
use crate::lexer::{AttributeName, Lexer, SyntaxError, Token};
use crate::pkg::{Extern, Public, Representation};

/// A collection of attributes in the source file.
#[derive(Default)]
pub(super) struct Attributes {
    public: Option<(AttributeName, Public)>,
    condition: Option<(AttributeName, Vec<Expression>)>,
    ext: Option<(AttributeName, Extern)>,
    repr: Option<(AttributeName, Representation)>,
    entry: Option<AttributeName>,
    customs: Vec<(AttributeName, Option<Vec<Vec<Expression>>>)>,
}

impl Attributes {
    pub fn parse(lex: &mut Lexer, first: AttributeName) -> Result<Self, SyntaxError> {
        // Parse the first attribute.
        let mut attrs = Self::default();

        attrs.parse_single(lex, first)?;

        // Parse the remaining if available.
        loop {
            match lex.next()? {
                Some(Token::AttributeName(name)) => attrs.parse_single(lex, name)?,
                Some(_) => {
                    lex.undo();
                    break;
                }
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap(),
                        "expected an item after this",
                    ));
                }
            }
        }

        Ok(attrs)
    }

    pub fn public(&self) -> Option<&(AttributeName, Public)> {
        self.public.as_ref()
    }

    pub fn ext(&self) -> Option<&(AttributeName, Extern)> {
        self.ext.as_ref()
    }

    pub fn repr(&self) -> Option<&(AttributeName, Representation)> {
        self.repr.as_ref()
    }

    pub fn entry(&self) -> Option<&AttributeName> {
        self.entry.as_ref()
    }

    pub fn run_condition(&self, cg: &Codegen) -> Result<bool, SyntaxError> {
        // Always return true if no condition.
        let cond = match &self.condition {
            Some(v) => &v.1,
            None => return Ok(true),
        };

        // Get first expression.
        let mut expr = cond.iter();
        let lhs = match expr.next().unwrap() {
            Expression::Value(v) => v,
            e => return Err(SyntaxError::new(e.span(), "expect an identifier")),
        };

        // Get second expression.
        let os = cg.target().os();
        let (equal, span) = match expr.next() {
            Some(Expression::NotEqual(f, s)) => (false, f.span() + s.span()),
            Some(Expression::Equal(f, s)) => (true, f.span() + s.span()),
            Some(e) => return Err(SyntaxError::new(e.span(), "unsupported expression")),
            None => {
                return Ok(if lhs.value() == "unix" {
                    os.is_unix()
                } else {
                    lhs.value() == os.name()
                })
            }
        };

        // Check if first expression is "os".
        if lhs.value() != "os" {
            return Err(SyntaxError::new(lhs.span().clone(), "unknown expression"));
        }

        // Get third argument.
        let rhs = match expr.next() {
            Some(Expression::String(v)) => v,
            Some(t) => return Err(SyntaxError::new(t.span(), "expect a string literal")),
            None => return Err(SyntaxError::new(span, "expect a string literal after this")),
        };

        // Compare.
        let res = if equal {
            rhs.value() == os.name()
        } else {
            rhs.value() != os.name()
        };

        if expr.next().is_some() {
            todo!()
        }

        Ok(res)
    }

    pub fn to_external(&self) -> crate::pkg::Attributes {
        crate::pkg::Attributes::new(
            self.public.as_ref().map(|v| v.1),
            self.ext.as_ref().map(|v| v.1),
            self.repr.as_ref().map(|v| v.1),
        )
    }

    fn parse_single(&mut self, lex: &mut Lexer, name: AttributeName) -> Result<(), SyntaxError> {
        match name.value() {
            "entry" => {
                // Check for multiple entry.
                if self.entry.is_some() {
                    return Err(SyntaxError::new(
                        name.span(),
                        "multiple entry attribute is not allowed",
                    ));
                }

                self.entry = Some(name);
            }
            "ext" => {
                // Check for multiple ext.
                if self.ext.is_some() {
                    return Err(SyntaxError::new(
                        name.span(),
                        "multiple ext attribute is not allowed",
                    ));
                }

                // Parse argument.
                lex.next_op()?;
                let ext = lex.next_ident()?;
                lex.next_cp()?;

                self.ext = Some((
                    name,
                    match ext.value() {
                        "C" => Extern::C,
                        _ => return Err(SyntaxError::new(ext.span(), "unknown extern")),
                    },
                ));
            }
            "if" => {
                // Check for multiple if.
                if self.condition.is_some() {
                    return Err(SyntaxError::new(
                        name.span(),
                        "multiple if attribute is not allowed",
                    ));
                }

                // Parse argument.
                lex.next_op()?;
                self.condition = Some((name, Expression::parse(lex)?));
                lex.next_cp()?;
            }
            "pub" => {
                // Check for multiple pub.
                if self.public.is_some() {
                    return Err(SyntaxError::new(
                        name.span(),
                        "multiple pub attribute is not allowed",
                    ));
                }

                // Parse argument.
                self.public = Some(match lex.next()? {
                    Some(Token::OpenParenthesis(_)) => match lex.next()? {
                        Some(Token::CloseParenthesis(_)) => (name, Public::External),
                        _ => todo!(),
                    },
                    Some(_) => {
                        lex.undo();
                        (name, Public::External)
                    }
                    None => (name, Public::External),
                });
            }
            "repr" => {
                // Check for multiple repr.
                if self.repr.is_some() {
                    return Err(SyntaxError::new(
                        name.span(),
                        "multiple repr attribute is not allowed",
                    ));
                }

                // Parse argument.
                lex.next_op()?;
                let repr = lex.next_ident()?;
                lex.next_cp()?;

                self.repr = Some((
                    name,
                    match repr.value() {
                        "i32" => Representation::I32,
                        "u8" => Representation::U8,
                        "un" => Representation::Un,
                        _ => return Err(SyntaxError::new(repr.span(), "unknown representation")),
                    },
                ));
            }
            v if v.chars().next().unwrap().is_ascii_lowercase() => {
                return Err(SyntaxError::new(
                    name.span(),
                    "an attribute begin with a lower case is a reserved name",
                ));
            }
            _ => self.customs.push((
                name,
                match lex.next()? {
                    Some(Token::OpenParenthesis(_)) => Some(Expression::parse_args(lex)?),
                    Some(Token::CloseParenthesis(v)) => {
                        return Err(SyntaxError::new(v.span(), "expect '('"));
                    }
                    _ => {
                        lex.undo();
                        None
                    }
                },
            )),
        }

        Ok(())
    }
}
