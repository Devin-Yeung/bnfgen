use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::error::{Error, Result};
use crate::span::Span;
use crate::utils::convert_parse_error;
use crate::{lexer, parser};

#[repr(transparent)]
#[derive(Debug)]
pub struct RawGrammar {
    pub(crate) rules: Vec<Rule>,
}

impl RawGrammar {
    pub fn parse<S: AsRef<str>>(input: S) -> Result<RawGrammar> {
        let lexer = lexer::Lexer::new(input.as_ref());
        let parser = parser::RawGrammarParser::new();
        parser.parse(lexer).map_err(convert_parse_error)
    }

    pub fn to_checked(self) -> Result<CheckedGrammar> {
        self.check_undefined()?.check_duplicate()?;

        let mut rules = HashMap::new();
        for rule in self.rules {
            rules.insert(rule.name, rule.production);
        }

        Ok(CheckedGrammar { rules })
    }

    fn check_duplicate(&self) -> Result<&Self> {
        let mut seen: HashMap<String, Span> = HashMap::new();
        for rule in &self.rules {
            if let Some(prev) = seen.get(&rule.name) {
                return Err(Error::DuplicatedRules {
                    span: rule.span,
                    prev: prev.clone(),
                });
            }
            seen.insert(rule.name.clone(), rule.span);
        }
        Ok(self)
    }
    fn check_undefined(&self) -> Result<&Self> {
        let defined: HashSet<String> =
            HashSet::from_iter(self.rules.iter().map(|r| r.name.clone()));
        for rule in &self.rules {
            for sym in rule
                .production
                .production
                .iter()
                .flat_map(|a| a.symbols.iter())
            {
                match &sym.kind {
                    SymbolKind::NonTerminal(s) => {
                        if !defined.contains(s.as_ref()) {
                            return Err(Error::UndefinedNonTerminal { span: sym.span });
                        }
                    }
                    _ => { /* do nothing */ }
                }
            }
        }
        Ok(self)
    }
}

pub struct CheckedGrammar {
    pub(crate) rules: HashMap<String, WeightedProduction>,
}

impl CheckedGrammar {
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
                    .expect(&format!("Fail to find rule of {}", s))
                    .choose(rng);
                (None, syms)
            }
            SymbolKind::Repeat { symbol, min, max } => {
                todo!()
            }
            SymbolKind::Regex(_) => {
                todo!()
            }
        }
    }
}

#[derive(Debug)]
pub struct Rule {
    pub(crate) name: String,
    pub(crate) production: WeightedProduction,
    pub(crate) span: Span,
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
pub(crate) enum SymbolKind {
    Terminal(Rc<String>),
    NonTerminal(Rc<String>),
    Regex(Rc<String>),
    Repeat {
        symbol: Box<SymbolKind>,
        min: usize,
        max: Option<usize>,
    },
}

#[derive(Debug)]
pub struct Symbol {
    pub(crate) kind: SymbolKind,
    pub(crate) span: Span,
}

#[cfg(test)]
mod test {
    use super::RawGrammar;
    use crate::report::{Reporter, Style};
    use miette::{Diagnostic, Report};
    use std::sync::Arc;

    fn report_with_unnamed_source<T: Diagnostic + Sync + Send + 'static, S: ToString>(
        err: T,
        source: S,
    ) -> String {
        let source = Arc::new(String::from(source.to_string()));
        let diagnostic = Report::from(err).with_source_code(source);

        let mut reporter = Reporter::new(Style::NoColor);
        reporter.push(diagnostic);
        reporter.report_to_string()
    }

    #[test]
    fn brainfuck() {
        let text = include_str!("../examples/brainfuck.bnfgen");
        let grammar = RawGrammar::parse(text).unwrap();
        insta::assert_debug_snapshot!(grammar);
    }

    #[test]
    fn unexpected_eof() {
        let text = "<start> ::= \"Hello\" | \"World\""; // no semi
        let err = RawGrammar::parse(text).err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn invalid_token() {
        let text = ":";
        let err = RawGrammar::parse(text).err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn undefined_nt() {
        let text = "<E> ::= <S>;";
        let err = RawGrammar::parse(text).unwrap().to_checked().err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn duplicated_def() {
        let text = r#"
            <E> ::= <S>;
            <S> ::= <E>;
            <E> ::= "?";
        "#;
        let err = RawGrammar::parse(text).unwrap().to_checked().err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }
}
