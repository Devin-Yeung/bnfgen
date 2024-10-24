use crate::grammar::symbol::Symbol;
use crate::span::Span;

#[derive(Debug)]
pub enum Limit {
    /// can be invoked any number of times
    Unlimited,
    Limited {
        /// should be invoked at least `min` times (inclusive)
        min: usize,
        /// should be invoked at most `max` times (inclusive)
        max: usize,
    },
}

#[derive(Debug)]
pub struct Alternative {
    pub(crate) span: Span,
    pub(crate) weight: usize,
    pub(crate) invoke_limit: Limit,
    pub(crate) symbols: Vec<Symbol>,
}

impl Alternative {
    pub(crate) fn non_re_terminals(&self) -> Vec<&str> {
        self.symbols
            .iter()
            .filter_map(|s| s.kind.non_re_terminal())
            .collect()
    }
}
