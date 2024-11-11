use crate::grammar::production::WeightedProduction;
use crate::grammar::state::State;
use crate::grammar::symbol::{NonTerminal, SymbolKind};
use rand::Rng;
use std::collections::HashMap;
use std::rc::Rc;

#[derive(Debug)]
pub struct CheckedGrammar {
    pub(crate) rules: HashMap<NonTerminal, WeightedProduction>,
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
                let syms = self
                    .rules
                    .get(&s)
                    .unwrap_or_else(|| panic!("Fail to find rule of {:?}", s))
                    .choose_by_state(state);

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
        insta::assert_debug_snapshot!(grammar.to_checked().unwrap());
    }
}
