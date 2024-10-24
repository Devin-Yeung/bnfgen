use crate::grammar::checked::CheckedGrammar;
use crate::grammar::symbol::SymbolKind;
use rand::Rng;

#[derive(typed_builder::TypedBuilder)]
pub struct Generator {
    pub grammar: CheckedGrammar,
}

impl Generator {
    pub fn generate<R: Rng, S: ToString>(&self, start: S, rng: &mut R) -> String {
        let mut buf = Vec::new();

        let start = SymbolKind::NonTerminal(start.to_string().into());
        let mut stack = vec![start];

        while !stack.is_empty() {
            // pop out the first symbol
            match self.grammar.reduce(stack.remove(0), rng) {
                (Some(s), syms) => {
                    buf.push(s);
                    stack.extend(syms);
                }
                (None, mut syms) => {
                    // syms :: stack
                    syms.extend(stack);
                    stack = syms;
                }
            }
        }

        buf.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
    }
}
