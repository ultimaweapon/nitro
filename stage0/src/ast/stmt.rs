use super::expr::Expression;
use super::Attributes;
use crate::lexer::{Identifier, LetKeyword, Lexer, SyntaxError, Token};

/// A statement.
pub(super) enum Statement {
    Let(Let),
    Unit(Vec<Expression>),
    Value(Vec<Expression>),
}

impl Statement {
    pub fn parse_block(lex: &mut Lexer) -> Result<Vec<Self>, SyntaxError> {
        let mut block = Vec::new();

        while let Some(stmt) = Self::parse(lex)? {
            block.push(stmt);
        }

        Ok(block)
    }

    fn parse(lex: &mut Lexer) -> Result<Option<Self>, SyntaxError> {
        // Parse attributes.
        let attrs = match lex.next()? {
            Some(Token::AttributeName(name)) => {
                let attrs = Attributes::parse(lex, name)?;

                // Make sure there are a statement after the attributes.
                match lex.next()? {
                    Some(Token::CloseCurly(v)) => {
                        return Err(SyntaxError::new(v.span().clone(), "expect a statement"));
                    }
                    Some(_) => lex.undo(),
                    None => {
                        return Err(SyntaxError::new(
                            lex.last().unwrap().clone(),
                            "expect a statement after this",
                        ));
                    }
                }

                attrs
            }
            Some(v) => {
                lex.undo();
                Attributes::default()
            }
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an '}' after this",
                ));
            }
        };

        // Parse statement.
        let stmt = match lex.next()? {
            Some(Token::LetKeyword(def)) => {
                let name = lex.next_ident()?;
                lex.next_equals()?;

                let exprs = Expression::parse(lex)?;
                lex.next_semicolon()?;

                Statement::Let(Let::new(attrs, def, name, exprs))
            }
            Some(Token::CloseCurly(_)) => return Ok(None),
            Some(_) => {
                lex.undo();

                let exprs = Expression::parse(lex)?;

                match lex.next()? {
                    Some(Token::Semicolon(_)) => Statement::Unit(exprs),
                    Some(Token::CloseCurly(_)) => {
                        lex.undo();
                        Statement::Value(exprs)
                    }
                    Some(t) => return Err(SyntaxError::new(t.span().clone(), "expect ';'")),
                    None => {
                        return Err(SyntaxError::new(
                            lex.last().unwrap().clone(),
                            "expect an '}' after this",
                        ));
                    }
                }
            }
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap().clone(),
                    "expect an '}' after this",
                ));
            }
        };

        Ok(Some(stmt))
    }
}

/// A let statement.
pub(super) struct Let {
    attrs: Attributes,
    def: LetKeyword,
    var: Identifier,
    val: Vec<Expression>,
}

impl Let {
    pub fn new(attrs: Attributes, def: LetKeyword, var: Identifier, val: Vec<Expression>) -> Self {
        Self {
            attrs,
            def,
            var,
            val,
        }
    }
}
