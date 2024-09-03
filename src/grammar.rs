use crate::{lexer, parser};

#[repr(transparent)]
#[derive(Debug)]
pub struct RawGrammar {
    pub(crate) rules: Vec<Rule>,
}

impl RawGrammar {
    pub fn parse<S: AsRef<str>>(input: S) -> RawGrammar {
        let lexer = lexer::Lexer::new(input.as_ref());
        let parser = parser::RawGrammarParser::new();
        parser.parse(lexer).unwrap() // TODO: Error Handling
    }
}

#[derive(Debug)]
pub struct Rule {
    pub(crate) name: String,
    pub(crate) alternatives: Vec<Alternative>,
}

#[derive(Debug)]
pub struct Alternative {
    pub(crate) weight: usize,
    pub(crate) symbols: Vec<Symbol>,
}

#[derive(Debug)]
pub enum SymbolKind {
    Terminal(String),
    NonTerminal(String),
    Repeat {
        symbol: Box<Symbol>,
        min: usize,
        max: Option<usize>,
    },
}

#[derive(Debug)]
pub struct Symbol {
    pub(crate) kind: SymbolKind,
}

#[cfg(test)]
mod test {
    use super::RawGrammar;

    #[test]
    fn brainfuck() {
        let text = include_str!("../examples/brainfuck.bnfgen");
        let grammar = RawGrammar::parse(text);
        insta::assert_debug_snapshot!(grammar);
    }
}
