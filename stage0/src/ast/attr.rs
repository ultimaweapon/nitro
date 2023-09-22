use super::Expression;
use crate::lexer::{AttributeName, Lexer, SyntaxError, Token};

/// A collection of attributes.
#[derive(Default)]
pub struct Attributes {
    public: Option<AttributeName>,
    condition: Option<(AttributeName, Vec<Expression>)>,
    ext: Option<(AttributeName, Extern)>,
    repr: Option<(AttributeName, Representation)>,
    customs: Vec<(AttributeName, Option<Vec<Vec<Expression>>>)>,
}

impl Attributes {
    pub fn parse(lex: &mut Lexer, first: AttributeName) -> Result<Self, SyntaxError> {
        // Parse the first attribute.
        let mut attrs = Self {
            public: None,
            condition: None,
            ext: None,
            repr: None,
            customs: Vec::new(),
        };

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
                        lex.last().unwrap().clone(),
                        "expected an item after this",
                    ));
                }
            }
        }

        Ok(attrs)
    }

    pub fn condition(&self) -> Option<&(AttributeName, Vec<Expression>)> {
        self.condition.as_ref()
    }

    pub fn ext(&self) -> Option<&(AttributeName, Extern)> {
        self.ext.as_ref()
    }

    pub fn repr(&self) -> Option<&(AttributeName, Representation)> {
        self.repr.as_ref()
    }

    fn parse_single(&mut self, lex: &mut Lexer, name: AttributeName) -> Result<(), SyntaxError> {
        match name.value() {
            "ext" => {
                // Check for multiple ext.
                if self.ext.is_some() {
                    return Err(SyntaxError::new(
                        name.span().clone(),
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
                        _ => return Err(SyntaxError::new(ext.span().clone(), "unknown extern")),
                    },
                ));
            }
            "if" => {
                // Check for multiple if.
                if self.condition.is_some() {
                    return Err(SyntaxError::new(
                        name.span().clone(),
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
                        name.span().clone(),
                        "multiple pub attribute is not allowed",
                    ));
                }

                // Parse argument.
                self.public = Some(match lex.next()? {
                    Some(Token::OpenParenthesis(_)) => match lex.next()? {
                        Some(Token::CloseParenthesis(_)) => name,
                        _ => todo!(),
                    },
                    Some(_) => {
                        lex.undo();
                        name
                    }
                    None => name,
                });
            }
            "repr" => {
                // Check for multiple repr.
                if self.repr.is_some() {
                    return Err(SyntaxError::new(
                        name.span().clone(),
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
                        "u8" => Representation::U8,
                        "un" => Representation::Un,
                        _ => {
                            return Err(SyntaxError::new(
                                repr.span().clone(),
                                "unknown representation",
                            ));
                        }
                    },
                ));
            }
            v if v.chars().next().unwrap().is_ascii_lowercase() => {
                return Err(SyntaxError::new(
                    name.span().clone(),
                    "an attribute begin with a lower case is a reserved name",
                ));
            }
            _ => self.customs.push((
                name,
                match lex.next()? {
                    Some(Token::OpenParenthesis(_)) => Some(Expression::parse_args(lex)?),
                    Some(Token::CloseParenthesis(v)) => {
                        return Err(SyntaxError::new(v.span().clone(), "expect '('"));
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

/// Argument of `@ext`.
pub enum Extern {
    C,
}

/// Argument of `@repr`
#[derive(Clone, Copy)]
pub enum Representation {
    U8,
    Un,
}
