use crate::grammar::state::State;
use crate::grammar::symbol::Symbol;
use crate::span::Span;
use rand::Rng;
use std::hash::{DefaultHasher, Hash, Hasher};

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

impl Hash for Alternative {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.symbols.hash(state);
    }
}

pub type AltId = u64;

impl Alternative {
    /// returns the non-regex terminals in this alternative
    pub(crate) fn non_re_terminals(&self) -> Vec<&str> {
        self.symbols
            .iter()
            .filter_map(|s| s.kind.non_re_terminal())
            .collect()
    }

    /// return the unique id of this alternative
    pub(crate) fn id(&self) -> AltId {
        let mut hasher = DefaultHasher::new();
        self.hash(&mut hasher);
        hasher.finish()
    }

    pub(crate) fn has_invoke_limits(&self) -> bool {
        !matches!(self.invoke_limit, Limit::Unlimited)
    }

    /// check if this alternative has exceeded its invoke limit base on the generator state
    pub(crate) fn exceeds_invoke_limit<R: Rng>(&self, state: &State<R>) -> bool {
        match self.invoke_limit {
            Limit::Unlimited => false,
            Limit::Limited { max, .. } => state.count(self.id()) > max,
        }
    }

    pub(crate) fn lose_invoke_limit<R: Rng>(&self, state: &State<R>) -> bool {
        match self.invoke_limit {
            Limit::Unlimited => false,
            Limit::Limited { min, .. } => state.count(self.id()) < min,
        }
    }
}
