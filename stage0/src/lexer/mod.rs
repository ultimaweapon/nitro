pub use self::span::*;
pub use self::token::*;

use std::borrow::Cow;
use std::error::Error;
use std::fmt::{Display, Formatter};
use std::rc::Rc;

mod span;
mod token;

/// Tokenizer for Nitro source.
pub struct Lexer {
    data: Rc<String>,
    next: usize,
    last: Option<Span>,
}

impl Lexer {
    pub fn new<D: Into<String>>(data: D) -> Self {
        Self {
            data: Rc::new(data.into()),
            next: 0,
            last: None,
        }
    }

    pub fn last(&self) -> Option<&Span> {
        self.last.as_ref()
    }

    pub fn next_equals(&mut self) -> Result<Equals, SyntaxError> {
        let tok = match self.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    self.last().unwrap().clone(),
                    "expect an '=' after this",
                ));
            }
        };

        match tok {
            Token::Equals(v) => Ok(v),
            t => Err(SyntaxError::new(t.span().clone(), "expect an '='")),
        }
    }

    pub fn next_colon(&mut self) -> Result<Colon, SyntaxError> {
        let tok = match self.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    self.last().unwrap().clone(),
                    "expect an ':' after this",
                ));
            }
        };

        match tok {
            Token::Colon(v) => Ok(v),
            t => Err(SyntaxError::new(t.span().clone(), "expect an ':'")),
        }
    }

    pub fn next_semicolon(&mut self) -> Result<Semicolon, SyntaxError> {
        let tok = match self.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    self.last().unwrap().clone(),
                    "expect an ';' after this",
                ));
            }
        };

        match tok {
            Token::Semicolon(v) => Ok(v),
            t => Err(SyntaxError::new(t.span().clone(), "expect an ';'")),
        }
    }

    pub fn next_op(&mut self) -> Result<OpenParenthesis, SyntaxError> {
        let tok = match self.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    self.last().unwrap().clone(),
                    "expect an '(' after this",
                ));
            }
        };

        match tok {
            Token::OpenParenthesis(v) => Ok(v),
            t => Err(SyntaxError::new(t.span().clone(), "expect an '('")),
        }
    }

    pub fn next_cp(&mut self) -> Result<CloseParenthesis, SyntaxError> {
        let tok = match self.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    self.last().unwrap().clone(),
                    "expect an ')' after this",
                ));
            }
        };

        match tok {
            Token::CloseParenthesis(v) => Ok(v),
            t => Err(SyntaxError::new(t.span().clone(), "expect an ')'")),
        }
    }

    pub fn next_oc(&mut self) -> Result<OpenCurly, SyntaxError> {
        let tok = match self.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    self.last().unwrap().clone(),
                    "expect an '{' after this",
                ));
            }
        };

        match tok {
            Token::OpenCurly(v) => Ok(v),
            t => Err(SyntaxError::new(t.span().clone(), "expect an '{'")),
        }
    }

    pub fn next_ident(&mut self) -> Result<Identifier, SyntaxError> {
        let tok = match self.next()? {
            Some(v) => v,
            None => {
                return Err(SyntaxError::new(
                    self.last().unwrap().clone(),
                    "expect an identifier after this",
                ));
            }
        };

        match tok {
            Token::Identifier(v) => Ok(v),
            t => Err(SyntaxError::new(t.span().clone(), "expect an identifier")),
        }
    }

    pub fn next(&mut self) -> Result<Option<Token>, SyntaxError> {
        // Find a non-whitespace.
        let mut iter = self.data[self.next..].chars();
        let ch = loop {
            let ch = match iter.next() {
                Some(v) => v,
                None => return Ok(None),
            };

            self.next += ch.len_utf8();

            if !ch.is_whitespace() {
                break ch;
            }
        };

        // Check if a punctuation.
        let span = Span::new(self.data.clone(), self.next - ch.len_utf8(), ch.len_utf8());
        let tok: Option<Token> = match ch {
            '!' => Some(ExclamationMark::new(span).into()),
            '=' => Some(Equals::new(span).into()),
            '*' => Some(Asterisk::new(span).into()),
            '.' => Some(FullStop::new(span).into()),
            ',' => Some(Comma::new(span).into()),
            ':' => Some(Colon::new(span).into()),
            ';' => Some(Semicolon::new(span).into()),
            '(' => Some(OpenParenthesis::new(span).into()),
            ')' => Some(CloseParenthesis::new(span).into()),
            '{' => Some(OpenCurly::new(span).into()),
            '}' => Some(CloseCurly::new(span).into()),
            _ => None,
        };

        if let Some(t) = tok {
            self.last = Some(t.span().clone());
            return Ok(Some(t));
        }

        // Check if a prefixed token.
        let tok = match ch {
            '@' => {
                let name = self.read(Self::is_ident);
                let span = Span::new(
                    self.data.clone(),
                    self.next - name.len() - ch.len_utf8(),
                    name.len() + ch.len_utf8(),
                );

                if name.is_empty() {
                    return Err(SyntaxError::new(span, "no attribute name is specified"));
                }

                AttributeName::new(span, name).into()
            }
            '"' => {
                let start = self.next - ch.len_utf8();
                let mut value = String::new();

                loop {
                    let ch = match iter.next() {
                        Some(v) => v,
                        None => {
                            return Err(SyntaxError::new(
                                Span::new(self.data.clone(), start, self.next - start),
                                "incomplete string",
                            ));
                        }
                    };

                    self.next += ch.len_utf8();

                    match ch {
                        '\n' => {
                            // We don't support multi-line string literal because new line is
                            // different on each platform.
                            return Err(SyntaxError::new(
                                Span::new(
                                    self.data.clone(),
                                    start,
                                    self.next - ch.len_utf8() - start,
                                ),
                                "incomplete string",
                            ));
                        }
                        '"' => break,
                        v => value.push(v),
                    }
                }

                StringLiteral::new(
                    Span::new(self.data.clone(), start, self.next - start),
                    value,
                )
                .into()
            }
            ch => {
                self.next -= ch.len_utf8();

                if ch.is_ascii_digit() {
                    let lit = self.read(|c| c.is_ascii_digit() || c == '.');
                    let span = Span::new(self.data.clone(), self.next - lit.len(), lit.len());
                    Self::parse_num(lit, span)?
                } else if Self::is_ident(ch) {
                    let ident = self.read(Self::is_ident);
                    let span = Span::new(self.data.clone(), self.next - ident.len(), ident.len());
                    Self::parse_ident(ident, span)?
                } else {
                    todo!()
                }
            }
        };

        self.last = Some(tok.span().clone());

        Ok(Some(tok))
    }

    pub fn undo(&mut self) {
        let last = self.last.take().unwrap();
        self.next = last.offset();
    }

    fn parse_num(lit: String, span: Span) -> Result<Token, SyntaxError> {
        let tok = if lit.contains('.') {
            match lit.parse() {
                Ok(v) => FloatLiteral::new(span, v).into(),
                Err(_) => return Err(SyntaxError::new(span, "invalid floating point literal")),
            }
        } else {
            match lit.parse() {
                Ok(v) => UnsignedLiteral::new(span, v).into(),
                Err(_) => return Err(SyntaxError::new(span, "invalid integer literal")),
            }
        };

        Ok(tok)
    }

    fn parse_ident(ident: String, span: Span) -> Result<Token, SyntaxError> {
        let tok = match ident.as_str() {
            "asm" => AsmKeyword::new(span).into(),
            "class" => ClassKeyword::new(span).into(),
            "fn" => FnKeyword::new(span).into(),
            "if" => IfKeyword::new(span).into(),
            "is" => IsKeyword::new(span).into(),
            "impl" => ImplKeyword::new(span).into(),
            "let" => LetKeyword::new(span).into(),
            "null" => NullKeyword::new(span).into(),
            "self" => SelfKeyword::new(span).into(),
            "struct" => StructKeyword::new(span).into(),
            "use" => UseKeyword::new(span).into(),
            _ => Identifier::new(span, ident).into(),
        };

        Ok(tok)
    }

    fn is_ident(ch: char) -> bool {
        ch.is_alphanumeric() || ch == '_'
    }

    fn read<P>(&mut self, p: P) -> String
    where
        P: Fn(char) -> bool,
    {
        let start = self.next;

        for ch in self.data[start..].chars() {
            if !p(ch) {
                break;
            }

            self.next += ch.len_utf8();
        }

        self.data[start..self.next].to_owned()
    }
}

/// Represents an error when [`Lexer::next()`] is failed.
#[derive(Debug)]
pub struct SyntaxError {
    span: Span,
    reason: Cow<'static, str>,
}

impl SyntaxError {
    pub fn new<S, R>(span: S, reason: R) -> Self
    where
        S: Into<Span>,
        R: Into<Cow<'static, str>>,
    {
        Self {
            span: span.into(),
            reason: reason.into(),
        }
    }
}

impl Error for SyntaxError {}

impl Display for SyntaxError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.reason.fmt(f)?;
        writeln!(f)?;
        self.span.fmt(f)?;
        Ok(())
    }
}
