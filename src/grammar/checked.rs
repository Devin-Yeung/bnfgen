use crate::grammar::production::WeightedProduction;
use crate::grammar::state::State;
use crate::grammar::symbol::Ty::Untyped;
use crate::grammar::symbol::{NonTerminal, SymbolKind, Ty};
use indexmap::IndexMap;
use rand::prelude::IndexedRandom;
use rand::Rng;
use std::rc::Rc;

#[derive(Debug)]
pub struct CheckedGrammar {
    pub(crate) rules: IndexMap<NonTerminal, WeightedProduction>,
}

pub enum ReduceOutput {
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
