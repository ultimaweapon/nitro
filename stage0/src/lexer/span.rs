use std::fmt::{Display, Formatter};
use std::rc::Rc;

/// A span in the source file.
#[derive(Debug, Clone)]
pub struct Span {
    source: Rc<String>,
    offset: usize,
    length: usize,
}

impl Span {
    pub fn new(source: Rc<String>, offset: usize, length: usize) -> Self {
        Self {
            source,
            offset,
            length,
        }
    }

    pub fn source(&self) -> &Rc<String> {
        &self.source
    }

    pub fn offset(&self) -> usize {
        self.offset
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let head = &self.source[..self.offset];
        let span = &self.source[self.offset..(self.offset + self.length)];
        let tail = &self.source[(self.offset + self.length)..];
        let mut lines = vec![String::new()];
        let mut col = 0;

        // Load lines from head.
        for ch in head.chars() {
            match ch {
                '\r' => {}
                '\n' => {
                    lines.push(String::new());
                    col = 0;
                }
                v => {
                    lines.last_mut().unwrap().push(v);
                    col += 1;
                }
            }
        }

        // Push span content.
        lines.last_mut().unwrap().push_str(span);

        // Load remaining line.
        let mut tail = tail.chars();

        while let Some(ch) = tail.next() {
            match ch {
                '\r' => {}
                '\n' => {
                    lines.push(String::new());
                    break;
                }
                v => lines.last_mut().unwrap().push(v),
            }
        }

        // Push a cursor.
        for _ in 0..col {
            lines.last_mut().unwrap().push(' ');
        }

        if self.length == 0 {
            lines.last_mut().unwrap().push('^');
        } else {
            for _ in 0..self.length {
                lines.last_mut().unwrap().push('^');
            }
        }

        // Write.
        for i in lines.len().checked_sub(10).unwrap_or(0)..(lines.len() - 1) {
            writeln!(f, "{:>5} | {}", i + 1, lines[i])?;
        }

        write!(f, "      | {}", lines.last().unwrap())?;

        Ok(())
    }
}
