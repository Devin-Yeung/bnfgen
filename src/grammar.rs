use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;
use std::collections::HashMap;

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
        match parser.parse(lexer) {
            Ok(grammar) => grammar,
            Err(e) => match e {
                lalrpop_util::ParseError::UnrecognizedToken { token, expected } => {
                    panic!("Unrecognized token: {:?}, expected: {:?}", token, expected)
                }
                lalrpop_util::ParseError::ExtraToken { token } => {
                    panic!("Extra token: {:?}", token)
                }
                lalrpop_util::ParseError::User { error } => panic!("User error: {:?}", error),
                lalrpop_util::ParseError::InvalidToken { location } => {
                    panic!("Invalid token: {:?}", location)
                }
                lalrpop_util::ParseError::UnrecognizedEof { location, expected } => {
                    panic!("Unrecognized EOF: {:?}, expected: {:?}", location, expected)
                }
            },
        } // TODO: Error Handling
    }
}

pub struct CheckedGrammar {
    pub(crate) rules: HashMap<String, WeightedProduction>,
}

impl CheckedGrammar {
    fn reduce(&self, symbol: SymbolKind) -> (Option<String>, Vec<SymbolKind>) {
        match symbol {
            SymbolKind::Terminal(s) => (Some(s), vec![]),
            SymbolKind::NonTerminal(s) => {
                let syms = self.rules.get(&s).unwrap().choose(&mut rand::thread_rng());
                (None, syms)
            }
            SymbolKind::Repeat { symbol, min, max } => {
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub struct Rule {
    pub(crate) name: String,
    pub(crate) production: WeightedProduction,
}

#[derive(Debug)]
#[repr(transparent)]
pub struct WeightedProduction {
    pub(crate) production: Vec<Alternative>,
}

impl WeightedProduction {
    pub fn choose<R: Rng>(&self, rng: &mut R) -> Vec<SymbolKind> {
        let dist = WeightedIndex::new(self.production.iter().map(|a| a.weight)).unwrap();
        let idx = dist.sample(rng);
        self.production[idx]
            .symbols
            .iter()
            .map(|s| s.kind.clone())
            .collect()
    }
}

#[derive(Debug)]
pub struct Alternative {
    pub(crate) weight: usize,
    pub(crate) symbols: Vec<Symbol>,
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Terminal(String),
    NonTerminal(String),
    Repeat {
        symbol: Box<SymbolKind>,
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
