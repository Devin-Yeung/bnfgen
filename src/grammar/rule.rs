use crate::grammar::alt::Alternative;
use crate::grammar::production::WeightedProduction;
use crate::grammar::symbol::NonTerminal;
use crate::span::Span;

#[derive(Debug)]
pub struct Rule {
    pub(crate) lhs: NonTerminal,
    pub(crate) production: WeightedProduction,
    pub(crate) span: Span,
}

impl Rule {
    pub fn rhs(&self) -> &[Alternative] {
        self.production.alts.as_slice()
    }

    pub fn produce_terminals(&self) -> bool {
        self.production
            .alts
            .iter()
            .any(|a| a.symbols.iter().all(|s| s.kind.is_terminal()))
    }
}
