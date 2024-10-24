use crate::grammar::production::WeightedProduction;
use crate::grammar::symbol::SymbolKind;
use rand::Rng;
use std::collections::HashMap;
use std::rc::Rc;

pub struct CheckedGrammar {
    pub(crate) rules: HashMap<String, WeightedProduction>,
}

impl CheckedGrammar {
    /// '+' --reduce--> '+'
    ///
    /// E   --reduce--> E, remaining: ['+', E]
    /// if E -> E '+' E
    pub(crate) fn reduce<R: Rng>(
        &self,
        symbol: SymbolKind,
        rng: &mut R,
    ) -> (Option<Rc<String>>, Vec<SymbolKind>) {
        match symbol {
            SymbolKind::Terminal(s) => (Some(s), vec![]),
            SymbolKind::NonTerminal(s) => {
                let syms = self
                    .rules
                    .get(s.as_ref())
                    .unwrap_or_else(|| panic!("Fail to find rule of {}", s))
                    .choose(rng);
                (None, syms)
            }
            SymbolKind::Regex(re) => {
                let terminals = self
                    .rules
                    .values()
                    .flat_map(|r| r.non_re_terminals())
                    .collect::<Vec<_>>();
                let s = re.generate(rng, terminals.as_slice());
                (Some(Rc::new(s)), vec![])
            }
        }
    }
}
