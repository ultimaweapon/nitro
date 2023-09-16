use crate::lexer::{Span, Token};

/// A path of identifier (e.g. `foo.bar.Foo`).
pub struct Path {
    components: Vec<Token>,
}

impl Path {
    pub fn new(components: Vec<Token>) -> Self {
        assert!(!components.is_empty());
        assert!(components.last().unwrap().is_identifier());

        for i in 0..components.len() {
            if i % 2 == 0 {
                assert!(components[i].is_identifier());
            } else {
                assert!(components[i].is_full_stop());
            }
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
}
