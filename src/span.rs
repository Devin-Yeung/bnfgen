use miette::SourceSpan;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct Span {
    start: usize,
    end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }
}

impl From<Span> for SourceSpan {
    fn from(val: Span) -> Self {
        SourceSpan::from(val.start..val.end)
    }
}

impl From<logos::Span> for Span {
    fn from(val: logos::Span) -> Self {
        Self {
            start: val.start,
            end: val.end,
        }
    }
}
