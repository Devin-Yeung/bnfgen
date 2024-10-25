use crate::grammar::production::WeightedProduction;
use crate::grammar::state::State;
use crate::grammar::symbol::SymbolKind;
use rand::Rng;
use std::collections::HashMap;
use std::rc::Rc;

pub struct CheckedGrammar {
    pub(crate) rules: HashMap<String, WeightedProduction>,
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
                    .get(s.as_ref())
                    .unwrap_or_else(|| panic!("Fail to find rule of {}", s))
                    .choose_by_state(state);

                ReduceOutput::NonTerminal { name: s, syms }
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
