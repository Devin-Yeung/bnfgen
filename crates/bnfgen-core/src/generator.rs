use crate::grammar::checked::{CheckedGrammar, ReduceOutput};
use crate::grammar::state::State;
use crate::grammar::symbol::SymbolKind::Terminal;
use crate::grammar::symbol::{NonTerminal, SymbolKind};
use crate::parse_tree::tree::ParseTree;
use crate::Result;
use rand::Rng;

/// Stack-based iterative string generator.
///
/// Generates random strings from a validated BNF grammar using an iterative
/// stack-based approach. The generator processes symbols by maintaining a stack
/// of symbols to expand, popping symbols and reducing them until only terminals
/// remain.
///
/// # Example
///
/// ```rust
/// use bnfgen::{RawGrammar, Generator, Result};
///
/// # fn main() -> Result<()> {
/// let grammar = RawGrammar::parse("<S> ::= \"hello\" | \"world\";")?;
/// let checked = grammar.to_checked()?;
/// let mut gen = Generator::new(checked);
/// let output = gen.generate("S", &mut rand::rng())?;
/// # Ok(())
/// # }
/// ```
pub struct Generator {
    grammar: CheckedGrammar,
}

impl Generator {
    /// Creates a new generator from a validated grammar.
    ///
    /// # Arguments
    ///
    /// * `grammar` - A validated `CheckedGrammar` obtained from `RawGrammar::to_checked()`
    pub fn new(grammar: CheckedGrammar) -> Self {
        Self { grammar }
    }

    /// Generates a random string starting from the given non-terminal.
    ///
    /// # Arguments
    ///
    /// * `start` - The name of the non-terminal to start generation from (e.g., "S")
    /// * `rng` - A random number generator implementing `rand::Rng`
    ///
    /// # Returns
    ///
    /// A space-separated string of terminals, or an error if generation fails
    /// (e.g., all alternatives for a non-terminal have exceeded their invoke limits).
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::{RawGrammar, Generator, Result};
    /// use rand::SeedableRng;
    ///
    /// # fn main() -> Result<()> {
    /// let grammar = RawGrammar::parse("<S> ::= \"a\" | \"b\";")?;
    /// let checked = grammar.to_checked()?;
    /// let gen = Generator::new(checked);
    /// let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    /// let output = gen.generate("S", &mut rng)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate<R: Rng, S: Into<String>>(&self, start: S, rng: &mut R) -> Result<String> {
        let mut buf = Vec::new();
        let mut state = State::new(rng);

        let start = SymbolKind::NonTerminal(NonTerminal::untyped(start));
        let mut stack = vec![start];

        while !stack.is_empty() {
            // pop out the first symbol
            match self.grammar.reduce(stack.remove(0), &mut state)? {
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

        Ok(buf.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(" "))
    }
}

/// Recursive tree-based generator that produces parse trees.
///
/// Unlike [`Generator`], which produces a flat string output, `TreeGenerator`
/// builds a full parse tree structure that can be used for analysis or
/// transformation of the generated output.
///
/// # Example
///
/// ```rust
/// use bnfgen::{RawGrammar, TreeGenerator, Result};
/// use rand::SeedableRng;
///
/// # fn main() -> Result<()> {
/// let grammar = RawGrammar::parse("<S> ::= \"hello\";")?;
/// let checked = grammar.to_checked()?;
/// let gen = TreeGenerator::new(checked);
/// let mut rng = rand::rngs::StdRng::seed_from_u64(42);
/// let tree = gen.generate("S", &mut rng);
/// # Ok(())
/// # }
/// ```
pub struct TreeGenerator {
    grammar: CheckedGrammar,
}

impl TreeGenerator {
    /// Creates a new tree generator from a validated grammar.
    ///
    /// # Arguments
    ///
    /// * `grammar` - A validated `CheckedGrammar` obtained from `RawGrammar::to_checked()`
    pub fn new(grammar: CheckedGrammar) -> Self {
        Self { grammar }
    }

    /// Generates a random parse tree starting from the given non-terminal.
    ///
    /// # Arguments
    ///
    /// * `start` - The name of the non-terminal to start generation from (e.g., "S")
    /// * `rng` - A random number generator implementing `rand::Rng`
    ///
    /// # Returns
    ///
    /// A [`ParseTree`] representing the generated structure, or an error if generation fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::{RawGrammar, TreeGenerator, Result};
    /// use rand::SeedableRng;
    ///
    /// # fn main() -> Result<()> {
    /// let grammar = RawGrammar::parse("<S> ::= \"a\" | \"b\";")?;
    /// let checked = grammar.to_checked()?;
    /// let gen = TreeGenerator::new(checked);
    /// let mut rng = rand::rngs::StdRng::seed_from_u64(42);
    /// let tree = gen.generate("S", &mut rng)?;
    /// # Ok(())
    /// # }
    /// ```
    pub fn generate<R: Rng, S: Into<String>>(
        &self,
        start: S,
        rng: &mut R,
    ) -> Result<ParseTree<SymbolKind>> {
        let start = SymbolKind::NonTerminal(NonTerminal::untyped(start));
        let mut state = State::new(rng);
        self.generate_tree(start, &mut state)
    }

    fn generate_tree<R: Rng>(
        &self,
        symbol: SymbolKind,
        state: &mut State<R>,
    ) -> Result<ParseTree<SymbolKind>> {
        match self.grammar.reduce(symbol, state)? {
            ReduceOutput::Terminal(s) => Ok(ParseTree::leaf(Terminal(s))),
            ReduceOutput::NonTerminal { name, syms } => {
                let subtrees = syms
                    .into_iter()
                    .map(|sym| self.generate_tree(sym, state))
                    .collect::<Result<Vec<_>>>()?;
                Ok(ParseTree::branch(name.to_string(), subtrees))
            }
        }
    }
}

#[cfg(test)]
mod test {
    use crate::generator::{Generator, TreeGenerator};
    use crate::grammar::raw::RawGrammar;
    use crate::Result;
    use rand::SeedableRng;

    #[test]
    fn repeat_works() {
        let text = r#"
            <S> ::= <E> | <S> <E> {100};
            <E> ::= "a" {1, 10} | "b" {2, } | "c" {3} | "fallback" ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let gen = Generator::new(grammar);
        let out = gen.generate("S", &mut rand::rng()).unwrap();
        assert!(out.split(" ").count() >= 100);
    }

    #[test]
    fn test_tree_generator() {
        let text = r#"
            <S> ::= <E> | <S> <E> {10};
            <E> ::= "a" {1, 10} | "b" {2, } | "c" {3} | "fallback" ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let tree_gen = TreeGenerator::new(grammar);
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
        let tree = tree_gen.generate("S", &mut seeded_rng).unwrap();
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
        let gen = Generator::new(grammar);
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
        insta::assert_snapshot!(gen.generate("S", &mut seeded_rng).unwrap());
    }

    #[test]
    fn test_typed_set_algebra_expr() {
        let text = include_str!("../examples/set-algebra-typed.bnfgen");
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let gen = Generator::new(grammar);
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
        let out = (0..100)
            .map(|_| gen.generate("Expr", &mut seeded_rng).unwrap())
            .collect::<Vec<_>>()
            .join("\n");
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_typed_set_algebra() {
        let text = include_str!("../examples/set-algebra-typed.bnfgen");
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let gen = Generator::new(grammar);
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
        let out = gen.generate("Program", &mut seeded_rng).unwrap();
        insta::assert_snapshot!(out);
    }

    #[test]
    fn test_no_candidates_error() {
        // This grammar demonstrates when all alternatives for a non-terminal
        // exceed their invoke limits during generation:
        // <S> must expand to <A> <A> <A> (exactly 3 times),
        // but <A> has only 2 alternatives, each with a limit of {0} (max 0).
        // After 2 invocations of <A>, no candidates remain for the third.
        let text = r#"
            <S> ::= <A> <A> <A> ;
            <A> ::= "a" {0} | "b" {0} ;
        "#;
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let gen = Generator::new(grammar);

        // This generation will always fail because <S> requires 3 invocations of <A>,
        // but <A> can only be invoked twice total (once for each alternative with max 0).
        // With max=0:
        // - count=0: 0 > 0 is false, not exceeded
        // - count=1: 1 > 0 is true, exceeded
        // So each alternative can only be selected once.
        // Try multiple seeds to find one that triggers the error
        let mut found_error = false;
        for seed in 0..100 {
            let mut rng = rand::rngs::StdRng::seed_from_u64(seed);
            match gen.generate("S", &mut rng) {
                Ok(_) => {}
                Err(err) => {
                    if let crate::Error::NoCandidatesAvailable { name, .. } = &err {
                        assert_eq!(name, "A");
                        found_error = true;
                        break;
                    }
                }
            }
        }
        assert!(
            found_error,
            "Expected to find at least one seed that triggers NoCandidatesAvailable error"
        );
    }

    #[test]
    fn test_core_ocaml() {
        let text = include_str!("../examples/core-ocaml.bnfgen");
        let grammar = RawGrammar::parse(text).unwrap().to_checked().unwrap();
        let gen = Generator { grammar };
        let mut seeded_rng = rand::rngs::StdRng::seed_from_u64(42);
        let out = (0..5)
            .map(|_| gen.generate("Expr", &mut seeded_rng))
            .collect::<Result<Vec<_>>>()
            .unwrap()
            .join("\n");
        insta::assert_snapshot!(out);
    }
}
