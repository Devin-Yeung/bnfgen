use crate::grammar::checked::{CheckedGrammar, ReduceOutput};
use crate::grammar::state::State;
use crate::grammar::symbol::SymbolKind::Terminal;
use crate::grammar::symbol::{NonTerminal, SymbolKind};
use crate::parse_tree::tree::ParseTree;
use rand::Rng;

#[derive(typed_builder::TypedBuilder)]
pub struct Generator {
    pub grammar: CheckedGrammar,
}

impl Generator {
    pub fn generate<R: Rng, S: Into<String>>(&self, start: S, rng: &mut R) -> String {
        let mut buf = Vec::new();
        let mut state = State::new(rng);

        let start = SymbolKind::NonTerminal(NonTerminal::untyped(start));
        let mut stack = vec![start];

        while !stack.is_empty() {
            // pop out the first symbol
            match self.grammar.reduce(stack.remove(0), &mut state) {
                ReduceOutput::Terminal(s) => {
                    buf.push(s);
                }
                ReduceOutput::NonTerminal { mut syms, .. } => {
                    // syms :: stack
                    syms.extend(stack);
                    stack = syms;
                }
            }
        }

        buf.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" ")
    }
}

pub struct TreeGenerator {
    pub grammar: CheckedGrammar,
}

impl TreeGenerator {
    pub fn generate<R: Rng, S: Into<String>>(
        &self,
        start: S,
        rng: &mut R,
    ) -> ParseTree<SymbolKind> {
        let start = SymbolKind::NonTerminal(NonTerminal::untyped(start));
        let mut state = State::new(rng);
        self.generate_tree(start, &mut state)
    }

    fn generate_tree<R: Rng>(
        &self,
        symbol: SymbolKind,
        state: &mut State<R>,
    ) -> ParseTree<SymbolKind> {
        match self.grammar.reduce(symbol, state) {
            ReduceOutput::Terminal(s) => ParseTree::leaf(Terminal(s)),
            ReduceOutput::NonTerminal { name, syms } => {
                let subtrees = syms
                    .into_iter()
                    .map(|sym| self.generate_tree(sym, state))
                    .collect::<Vec<_>>();
                ParseTree::branch(name.to_string(), subtrees)
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::generator::{Generator, TreeGenerator};
    use crate::grammar::raw::RawGrammar;
    use rand::SeedableRng;

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

    #[test]
    fn test_tree_generator() {
        let text = r#"
            <S> ::= <E> | <S> <E> {10};
            <E> ::= "a" {1, 10} | "b" {2, } | "c" {3} | "fallback" ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let tree_gen = TreeGenerator { grammar };
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
        let tree = tree_gen.generate("S", &mut seeded_rng);
        insta::assert_debug_snapshot!(&tree);
    }

    #[test]
    fn test_typed_generator() {
        let text = r#"
            <S> ::= <Expr> | <S> "\n" <Expr> {10, 20};

            <Expr> ::= <E> ;

            <E: "int">  ::= "1" | "2" | "3"
                            | <E: "int"> "+" <E: "int"> {3, } ;

            <E: "bool"> ::= "true" | "false"
                            | <E: "bool"> "&" <E: "bool"> {3, } ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let gen = Generator { grammar };
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
        insta::assert_snapshot!(gen.generate("S", &mut seeded_rng));
    }

    #[test]
    fn test_typed_set_algebra() {
        let text = include_str!("../examples/set-algebra-typed.bnfgen");
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let gen = Generator { grammar };
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
        insta::assert_snapshot!(gen.generate("Program", &mut seeded_rng));
    }
}
