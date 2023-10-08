use super::{Attributes, Path};
use crate::lexer::{Identifier, Lexer, SyntaxError, Token, UseKeyword};

/// A `use` declaration.
pub struct Use {
    attrs: Attributes,
    def: UseKeyword,
    name: Path,
    rename: Option<Identifier>,
}

impl Use {
    pub fn parse(lex: &mut Lexer, attrs: Attributes, def: UseKeyword) -> Result<Self, SyntaxError> {
        // Get the package name.
        let mut name = Vec::new();

        match lex.next()? {
            Some(Token::SelfKeyword(v)) => name.push(Token::SelfKeyword(v)),
            Some(Token::Identifier(v)) => name.push(Token::Identifier(v)),
            Some(t) => {
                return Err(SyntaxError::new(
                    t.span(),
                    "expect an identifer or self keyword",
                ));
            }
            None => {
                return Err(SyntaxError::new(
                    def.span(),
                    "expect an identifer or self keyword after this",
                ));
            }
        }

        match lex.next()? {
            Some(Token::FullStop(v)) => name.push(Token::FullStop(v)),
            Some(t) => return Err(SyntaxError::new(t.span(), "expect '.'")),
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap(),
                    "expect '.' after this",
                ));
            }
        }

        // Get item after the package name.
        match lex.next()? {
            Some(Token::Identifier(v)) => name.push(Token::Identifier(v)),
            Some(t) => return Err(SyntaxError::new(t.span(), "expect an identifer")),
            None => {
                return Err(SyntaxError::new(
                    lex.last().unwrap(),
                    "expect an identifer after this",
                ));
            }
        }

        // Get remaining path.
        loop {
            let next = match lex.next()? {
                Some(v) => v,
                None => {
                    return Err(SyntaxError::new(
                        lex.last().unwrap(),
                        "expect ';' after this",
                    ));
                }
            };

            match next {
                Token::FullStop(v) => {
                    name.push(Token::FullStop(v));

                    match lex.next()? {
                        Some(Token::Identifier(v)) => name.push(Token::Identifier(v)),
                        Some(t) => return Err(SyntaxError::new(t.span(), "expect an identifer")),
                        None => {
                            return Err(SyntaxError::new(
                                lex.last().unwrap(),
                                "expect an identifer after this",
                            ));
                        }
                    }
                }
                Token::Semicolon(_) => break,
                t => return Err(SyntaxError::new(t.span(), "expect ';'")),
            }
        }

        Ok(Self {
            attrs,
            def,
            name: Path::new(name),
            rename: None,
        })
    }

    pub fn name(&self) -> &Path {
        &self.name
    }

    pub fn rename(&self) -> Option<&Identifier> {
        self.rename.as_ref()
    }
}
