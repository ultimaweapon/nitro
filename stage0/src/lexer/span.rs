use std::cmp::{max, min};
use std::fmt::{Display, Formatter};
use std::ops::Add;
use std::rc::Rc;

/// A span in the source file.
#[derive(Debug, Clone)]
pub struct Span {
    source: Rc<String>,
    begin: usize,
    end: usize,
}

impl Span {
    pub fn new(source: Rc<String>, offset: usize, length: usize) -> Self {
        assert_ne!(*source.as_bytes().get(offset).unwrap(), b'\n');
        assert_ne!(length, 0);

        Self {
            source,
            begin: offset,
            end: offset + length,
        }
    }

    pub fn source(&self) -> &Rc<String> {
        &self.source
    }

    pub fn offset(&self) -> usize {
        self.begin
    }

    fn create_indicator_line(target: &str, start: usize, end: usize) -> String {
        let mut target = target.chars();
        let mut line = String::new();

        for _ in 0..start {
            target.next().unwrap();
            line.push(' ');
        }

        for _ in start..end {
            line.push(if target.next().unwrap().is_whitespace() {
                ' '
            } else {
                '^'
            });
        }

        line
    }
}

impl From<&Self> for Span {
    fn from(value: &Self) -> Self {
        value.clone()
    }
}

impl Add for &Span {
    type Output = Span;

    fn add(self, rhs: Self) -> Self::Output {
        assert!(Rc::ptr_eq(&self.source, &rhs.source));

        let source = self.source.clone();
        let begin = min(self.begin, rhs.begin);
        let end = max(self.end, rhs.end);

        Span { source, begin, end }
    }
}

impl Display for Span {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let mut line = 0;
        let mut col = 0;
        let mut lines = vec![(String::new(), Some(line))];
        let mut offset = 0;
        let mut start = None;
        let mut end = None;
        let mut first = None;
        let mut last = None;

        for ch in self.source.chars() {
            if offset == self.begin {
                start = Some(col);
                first = Some(lines.len() - 1);
            } else if offset == self.end {
                end = Some(col);
            }

            match ch {
                '\r' => {}
                '\n' => {
                    if let Some(c) = start {
                        // Add an indicator line.
                        let l = lines.last().unwrap().0.as_str();
                        let e = end.unwrap_or_else(|| l.len());
                        let l = Self::create_indicator_line(l, c, e);

                        if l.chars().any(|c| !c.is_whitespace()) {
                            last = Some(lines.len());
                            lines.push((l, None));
                        }

                        // Check for multi-line span.
                        if end.is_some() {
                            start = None;
                            end = None;
                        } else {
                            start = Some(0);
                        }
                    }

                    // Insert next source line.
                    line += 1;
                    lines.push((String::new(), Some(line)));
                    col = 0;
                }
                _ => {
                    lines.last_mut().unwrap().0.push(ch);
                    col += 1;
                }
            }

            offset += ch.len_utf8();
        }

        if let Some(c) = start {
            let l = lines.last().unwrap().0.as_str();
            let e = l.len();
            let l = Self::create_indicator_line(l, c, e);

            if l.chars().any(|c| !c.is_whitespace()) {
                last = Some(lines.len());
                lines.push((l, None));
            }
        }

        // Write.
        let first = first.unwrap();
        let last = last.unwrap();

        for i in first..=last {
            let l = &lines[i];

            if let Some(n) = l.1 {
                // Line from the source is never be the last line.
                writeln!(f, "{:>5} | {}", n + 1, l.0)?;
            } else if i == last {
                write!(f, "      | {}", l.0)?;
            } else {
                writeln!(f, "      | {}", l.0)?;
            }
        }

        Ok(())
    }
}
