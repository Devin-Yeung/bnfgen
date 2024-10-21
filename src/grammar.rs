use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::Dfs;
use rand::distributions::Distribution;
use rand::distributions::WeightedIndex;
use rand::Rng;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;

use crate::error::{Error, Result};
use crate::regex::Regex;
use crate::span::Span;
use crate::utils::convert_parse_error;
use crate::{lexer, parser};

#[repr(transparent)]
#[derive(Debug)]
pub struct RawGrammar {
    pub(crate) rules: Vec<Rule>,
}

pub struct GrammarGraph<'rule> {
    pub(crate) rules: &'rule Vec<Rule>,
    pub(crate) graph: DiGraph<String, ()>,
    pub(crate) nodes: HashMap<String, NodeIndex>,
}

impl<'rule> GrammarGraph<'rule> {
    pub fn check_unused<S: AsRef<str>>(&self, start: S) -> Result<&Self> {
        let all_nts = self.nodes.keys().collect::<HashSet<_>>();
        // find the reachable nodes for a given start symbol
        let start = self
            .nodes
            .get(start.as_ref())
            .expect("The start symbol does not exist");

        let mut dfs = Dfs::new(&self.graph, *start);
        let mut reachable = HashSet::new();
        while let Some(nx) = dfs.next(&self.graph) {
            let name = &self.graph[nx];
            reachable.insert(name);
        }
        let unreachable = all_nts.difference(&reachable).collect::<HashSet<_>>();
        // find the unreachable spans
        if !unreachable.is_empty() {
            let spans = self
                .rules
                .iter()
                .filter(|rule| unreachable.contains(&&rule.name))
                .map(|rule| rule.span)
                .collect::<Vec<_>>();
            return Err(Error::UnreachableRules { spans });
        }
        Ok(self)
    }

    pub fn check_trap_loop(&self) -> Result<&Self> {
        let sccs = petgraph::algo::tarjan_scc(&self.graph);
        for scc in sccs {
            if self.is_trap_loop(&scc) {
                let spans = scc
                    .iter()
                    .map(|nx| {
                        self.rules
                            .iter()
                            .find(|rule| rule.name == self.graph[*nx])
                            .unwrap()
                            .span
                    })
                    .collect::<Vec<_>>();
                return Err(Error::TrapLoop { spans });
            }
        }
        Ok(&self)
    }

    fn is_trap_loop(&self, scc: &Vec<NodeIndex>) -> bool {
        let produce_t = scc.iter().map(|nx| self.graph[*nx].as_str()).any(|name| {
            // check if rule produce a terminal
            self.rules
                .iter()
                .find(|rule| rule.name == name)
                .unwrap()
                .produce_terminals()
        });
        if produce_t {
            return false;
        }
        let out_deg: HashSet<NodeIndex> = scc
            .iter()
            .map(|nx| self.graph.neighbors(*nx))
            .flatten()
            .collect();
        out_deg == scc.iter().map(|n| *n).collect()
    }
}

impl RawGrammar {
    pub fn parse<S: AsRef<str>>(input: S) -> Result<RawGrammar> {
        let lexer = lexer::Lexer::new(input.as_ref());
        let parser = parser::RawGrammarParser::new();
        parser.parse(lexer).map_err(convert_parse_error)
    }

    pub fn to_checked(self) -> Result<CheckedGrammar> {
        self.check_undefined()?.check_duplicate()?.check_repeats()?;

        let mut rules = HashMap::new();
        for rule in self.rules {
            rules.insert(rule.name, rule.production);
        }

        Ok(CheckedGrammar { rules })
    }

    pub fn graph(&self) -> GrammarGraph<'_> {
        let mut graph = DiGraph::<String, ()>::new();
        let nodes: HashMap<String, NodeIndex> = self
            .rules
            .iter()
            .map(|rule| {
                let entry = (rule.name.clone(), graph.add_node(rule.name.clone()));
                entry
            })
            .collect();
        // setup the graph
        for rule in &self.rules {
            for sym in rule.rhs().iter().flat_map(|a| a.symbols.iter()) {
                match sym.non_terminal() {
                    Some(name) => {
                        graph.add_edge(nodes[&rule.name], nodes[name], ());
                    }
                    None => { /* do nothing */ }
                }
            }
        }
        GrammarGraph {
            rules: &self.rules,
            graph,
            nodes,
        }
    }

    pub fn check_duplicate(&self) -> Result<&Self> {
        let mut seen: HashMap<String, Span> = HashMap::new();
        for rule in &self.rules {
            if let Some(prev) = seen.get(&rule.name) {
                return Err(Error::DuplicatedRules {
                    span: rule.span,
                    prev: *prev,
                });
            }
            seen.insert(rule.name.clone(), rule.span);
        }
        Ok(self)
    }

    pub fn check_repeats(&self) -> Result<&Self> {
        for rule in &self.rules {
            for sym in rule.rhs().iter().flat_map(|a| a.symbols.iter()) {
                match &sym.kind {
                    SymbolKind::Repeat {
                        symbol: _,
                        min,
                        max,
                    } => {
                        if min > &max.unwrap_or(usize::MAX) {
                            return Err(Error::InvalidRepeatRange { span: sym.span });
                        }
                    }
                    _ => { /* do nothing */ }
                }
            }
        }
        Ok(self)
    }

    pub fn check_undefined(&self) -> Result<&Self> {
        let defined: HashSet<String> =
            HashSet::from_iter(self.rules.iter().map(|r| r.name.clone()));
        for rule in &self.rules {
            for sym in rule.rhs().iter().flat_map(|a| a.symbols.iter()) {
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
                    .unwrap_or_else(|| panic!("Fail to find rule of {}", s))
                    .choose(rng);
                (None, syms)
            }
            SymbolKind::Repeat { symbol, min, max } => match (min, max) {
                (min, Some(max)) => {
                    if min == max {
                        (None, (0..=min).map(|_| *symbol.clone()).collect::<Vec<_>>())
                    } else {
                        todo!()
                    }
                }
                _ => todo!(),
            },
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

#[derive(Debug)]
pub struct Rule {
    pub(crate) name: String,
    pub(crate) production: WeightedProduction,
    pub(crate) span: Span,
}

impl Rule {
    pub fn rhs(&self) -> &[Alternative] {
        self.production.alts.as_slice()
    }

    pub fn produce_terminals(&self) -> bool {
        self.production
            .alts
            .iter()
            .any(|a| a.symbols.iter().all(|s| s.kind.is_terminal()))
    }
}

#[derive(Debug)]
#[repr(transparent)]
pub struct WeightedProduction {
    pub(crate) alts: Vec<Alternative>,
}

impl WeightedProduction {
    pub fn choose<R: Rng>(&self, rng: &mut R) -> Vec<SymbolKind> {
        let dist = WeightedIndex::new(self.alts.iter().map(|a| a.weight)).unwrap();
        let idx = dist.sample(rng);
        self.alts[idx]
            .symbols
            .iter()
            .map(|s| s.kind.clone())
            .collect()
    }

    pub fn non_re_terminals(&self) -> Vec<&str> {
        self.alts
            .iter()
            .flat_map(|a| a.non_re_terminals())
            .collect()
    }
}

#[derive(Debug)]
pub struct Alternative {
    pub(crate) weight: usize,
    pub(crate) symbols: Vec<Symbol>,
}

impl Alternative {
    pub(crate) fn non_re_terminals(&self) -> Vec<&str> {
        self.symbols
            .iter()
            .filter_map(|s| s.kind.non_re_terminal())
            .collect()
    }
}

#[derive(Debug, Clone)]
pub(crate) enum SymbolKind {
    Terminal(Rc<String>),
    NonTerminal(Rc<String>),
    Regex(Rc<Regex>),
    Repeat {
        symbol: Box<SymbolKind>,
        min: usize,
        max: Option<usize>,
    },
}

impl SymbolKind {
    pub fn non_re_terminal(&self) -> Option<&str> {
        match self {
            SymbolKind::Terminal(s) => Some(s.as_str()),
            SymbolKind::Repeat { symbol, .. } => symbol.non_re_terminal(),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        match self {
            SymbolKind::Terminal(_) | SymbolKind::Regex(_) => true,
            SymbolKind::Repeat { symbol, .. } => symbol.is_terminal(),
            _ => false,
        }
    }

    pub fn non_terminal(&self) -> Option<&str> {
        match self {
            SymbolKind::NonTerminal(s) => Some(s.as_str()),
            SymbolKind::Repeat { symbol, .. } => symbol.non_terminal(),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Symbol {
    pub(crate) kind: SymbolKind,
    pub(crate) span: Span,
}

impl Symbol {
    pub fn is_terminal(&self) -> bool {
        self.kind.is_terminal()
    }

    pub fn non_terminal(&self) -> Option<&str> {
        self.kind.non_terminal()
    }
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
        let source = Arc::new(source.to_string());
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
    fn repeat() {
        let text = r#"
            <E> ::= "a" {1, 10} | "b" {2, } | "c" {3} ;
        "#;
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
    fn invalid_re() {
        let text = r#"<R> ::= re("["); "#;
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

    #[test]
    fn invalid_repeat() {
        let text = r#"
            <E> ::= "a" {10, 1};
        "#;
        let err = RawGrammar::parse(text).unwrap().to_checked().err().unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn unreachable_nt() {
        let text = r#"
            <E> ::= "Hello" | <A> ;
            <W> ::= "World" ;
            <A> ::= <B> ;
            <B> ::= <A> ;
            <C> ::= <W> ;
        "#;
        let err = RawGrammar::parse(text)
            .unwrap()
            .graph()
            .check_unused("E")
            .err()
            .unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn trap_loop() {
        let text = r#"
            <E> ::= <D> | <F>;
            <C> ::= <D> ;
            <D> ::= <C> ;
            <F> ::= <G> ;
            <G> ::= <F> | "Terminal" ;
        "#;
        let err = RawGrammar::parse(text)
            .unwrap()
            .graph()
            .check_trap_loop()
            .err()
            .unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }

    #[test]
    fn tri_loop() {
        let text = r#"
            <no-trap-01> ::= <A> | "Terminal" ;
            <no-trap-02> ::= <A> | <term> ;
            <term> ::= "Terminal" ;
            <A> ::= <B> ;
            <B> ::= <C> ;
            <C> ::= <A> ;
        "#;
        let err = RawGrammar::parse(text)
            .unwrap()
            .graph()
            .check_trap_loop()
            .err()
            .unwrap();
        let ui = report_with_unnamed_source(err, text);
        insta::assert_snapshot!(ui);
    }
}
