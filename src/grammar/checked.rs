//! Validated grammar ready for generation.
//!
//! This module provides [`CheckedGrammar`], a validated grammar representation
//! suitable for use with generators. Obtain a `CheckedGrammar` by calling
//! [`RawGrammar::to_checked()`](super::raw::RawGrammar::to_checked).

use crate::grammar::production::WeightedProduction;
use crate::grammar::state::State;
use crate::grammar::symbol::Ty::Untyped;
use crate::grammar::symbol::{NonTerminal, SymbolKind, Ty};
use indexmap::IndexMap;
use rand::prelude::IndexedRandom;
use rand::Rng;
use std::rc::Rc;

/// A validated BNF grammar ready for generation.
///
/// `CheckedGrammar` is the output of validation checks performed by
/// [`RawGrammar::to_checked()`](super::raw::RawGrammar::to_checked).
/// It contains validated rules indexed by non-terminal and is used by
/// [`Generator`](crate::Generator) and [`TreeGenerator`](crate::TreeGenerator)
/// to produce random strings.
///
/// # Example
///
/// ```rust
/// use bnfgen::RawGrammar;
///
/// let grammar = RawGrammar::parse("<S> ::= \"a\" | \"b\";").unwrap();
/// let checked = grammar.to_checked().unwrap();
/// // checked is now ready for use with Generator
/// ```
#[derive(Debug)]
pub struct CheckedGrammar {
    pub(crate) rules: IndexMap<NonTerminal, WeightedProduction>,
}

pub(crate) enum ReduceOutput {
    Terminal(Rc<String>),
    NonTerminal {
        name: Rc<String>,
        syms: Vec<SymbolKind>,
    },
}

impl CheckedGrammar {
    /// '+' --reduce--> '+'
    ///
    /// E   --reduce--> E, remaining: ['+', E]
    /// if E -> E '+' E
    pub(crate) fn reduce<R: Rng>(&self, symbol: SymbolKind, state: &mut State<R>) -> ReduceOutput {
        match symbol {
            SymbolKind::Terminal(s) => ReduceOutput::Terminal(s),
            SymbolKind::NonTerminal(s) => {
                let syms = match s.ty {
                    Untyped => {
                        let candidates = self
                            .rules
                            .keys()
                            .filter(|k| k.name == s.name)
                            .collect::<Vec<_>>();
                        self.rules
                            .get(
                                *candidates
                                    .choose(state.rng())
                                    .expect("No candidates available"),
                            )
                            .unwrap_or_else(|| panic!("Fail to find rule of {:?}", s))
                            .choose_by_state(state)
                    }
                    Ty::Typed(_) => {
                        // require an exact match
                        self.rules
                            .get(&s)
                            .unwrap_or_else(|| panic!("Fail to find rule of {:?}", s))
                            .choose_by_state(state)
                    }
                };

                ReduceOutput::NonTerminal { name: s.name, syms }
            }
            SymbolKind::Regex(re) => {
                let terminals = self
                    .rules
                    .values()
                    .flat_map(|r| r.non_re_terminals())
                    .collect::<Vec<_>>();
                let s = re.generate(state.rng(), terminals.as_slice());
                ReduceOutput::Terminal(Rc::new(s))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::grammar::raw::RawGrammar;

    #[test]
    fn it_can_merge() {
        let text = r#"
            <E> ::= <E: "int"> "+" <E: "int"> ;
            <E> ::= <E: "str"> "+" <E: "str"> ;
            <E: "str"> ::= <E: "str"> "+" <E: "str"> ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap();
        assert!(grammar.to_checked().is_ok());
    }
}
