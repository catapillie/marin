use std::ops::Range;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Copy, Clone)]
pub struct Loc {
    pub span: Span,
    pub file: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    pub fn at(pos: usize) -> Self {
        Self {
            start: pos,
            end: pos,
        }
    }

    pub fn wrap(self, file: usize) -> Loc {
        Loc::new(self, file)
    }

    pub fn combine(left: Self, right: Self) -> Self {
        Self {
            start: usize::min(left.start, right.start),
            end: usize::max(left.end, right.end),
        }
    }

    pub fn lexeme<'a>(&self, source: &'a str) -> &'a str {
        &source[self.start..self.end]
    }
}

impl Default for Span {
    fn default() -> Self {
        Self {
            start: usize::MAX,
            end: usize::MIN,
        }
    }
}

impl From<Span> for Range<usize> {
    fn from(val: Span) -> Self {
        val.start..val.end
    }
}

impl From<Range<usize>> for Span {
    fn from(value: Range<usize>) -> Self {
        Self {
            start: value.start,
            end: value.end,
        }
    }
}

impl Loc {
    pub fn new(span: Span, file: usize) -> Self {
        Self { span, file }
    }
}
