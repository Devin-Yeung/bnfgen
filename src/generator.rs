use crate::grammar::checked::CheckedGrammar;
use crate::grammar::state::State;
use crate::grammar::symbol::SymbolKind;
use rand::Rng;

#[derive(typed_builder::TypedBuilder)]
pub struct Generator {
    pub grammar: CheckedGrammar,
}

impl Generator {
    pub fn generate<R: Rng, S: ToString>(&self, start: S, rng: &mut R) -> String {
        let mut buf = Vec::new();
        let mut state = State::new(rng);

        let start = SymbolKind::NonTerminal(start.to_string().into());
        let mut stack = vec![start];

        while !stack.is_empty() {
            // pop out the first symbol
            match self.grammar.reduce(stack.remove(0), &mut state) {
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

#[cfg(test)]
mod test {
    use crate::generator::Generator;
    use crate::grammar::raw::RawGrammar;

    #[test]
    fn repeat_works() {
        let text = r#"
            <S> ::= <E> | <S> <E> {100};
            <E> ::= "a" {1, 10} | "b" {2, } | "c" {3} | "fallback" ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let gen = Generator::builder().grammar(grammar).build();
        let out = gen.generate("S", &mut rand::thread_rng());
        assert!(out.split(" ").count() >= 100);
    }
}
