use crate::lexer::{Identifier, Span, Token};
use std::fmt::{Display, Formatter};

/// A path of identifier (e.g. `foo.bar.Foo`).
pub(super) struct Path {
    components: Vec<Token>,
}

impl Path {
    pub fn new(components: Vec<Token>) -> Self {
        // Requires at least one component and the last one must be identifier.
        assert!(!components.is_empty());
        assert!(components.last().unwrap().is_identifier());

        // Check the first component.
        let mut iter = components.iter();
        let mut next = iter.next().unwrap();

        assert!(next.is_self() || next.is_identifier());

        // Check the remaining components.
        let mut ident = false;

        for c in iter {
            if ident {
                assert!(c.is_identifier());
            } else {
                assert!(c.is_full_stop());
            }

            ident = !ident;
        }

        Self { components }
    }

    pub fn span(&self) -> Span {
        let mut iter = self.components.iter();
        let mut span = iter.next().unwrap().span().clone();

        for s in iter {
            span = &span + s.span();
        }

        span
    }

    pub fn as_local(&self) -> Option<&Identifier> {
        if self.components.len() == 1 {
            match &self.components[0] {
                Token::Identifier(v) => Some(v),
                _ => unreachable!(),
            }
        } else {
            None
        }
    }

    pub fn last(&self) -> &Identifier {
        match self.components.last().unwrap() {
            Token::Identifier(v) => v,
            _ => unreachable!(),
        }
    }
}

impl Display for Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        for c in &self.components {
            c.fmt(f)?;
        }

        Ok(())
    }
}
