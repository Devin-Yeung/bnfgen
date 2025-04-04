use crate::error::Error;
use crate::grammar::alt::Limit;
use crate::grammar::checked::CheckedGrammar;
use crate::grammar::graph::GrammarGraph;
use crate::grammar::rule::Rule;
use crate::grammar::symbol::SymbolKind;
use crate::lexer;
use crate::utils::convert_parse_error;
use indexmap::IndexMap;
use petgraph::graph::{DiGraph, NodeIndex};
use std::collections::{HashMap, HashSet};

#[repr(transparent)]
#[derive(Debug)]
pub struct RawGrammar {
    pub(crate) rules: Vec<Rule>,
}

impl RawGrammar {
    pub fn parse<S: AsRef<str>>(input: S) -> crate::error::Result<RawGrammar> {
        let lexer = lexer::Lexer::new(input.as_ref());
        let parser = crate::parser::RawGrammarParser::new();
        parser.parse(lexer).map_err(convert_parse_error)
    }

    pub fn to_checked(self) -> crate::error::Result<CheckedGrammar> {
        self.check_undefined()?.check_duplicate()?.check_repeats()?;

        let mut rules = IndexMap::new();
        for rule in self.rules {
            rules.insert(rule.lhs, rule.production);
        }

        Ok(CheckedGrammar { rules })
    }

    pub fn graph(&self) -> GrammarGraph<'_> {
        let mut graph = DiGraph::<String, ()>::new();
        let nodes: HashMap<String, NodeIndex> = self
            .rules
            .iter()
            .map(|rule| {
                (
                    rule.lhs.as_str().to_string(),
                    graph.add_node(rule.lhs.as_str().to_string()),
                )
            })
            .collect();
        // setup the graph
        for rule in &self.rules {
            for sym in rule.rhs().iter().flat_map(|a| a.symbols.iter()) {
                match sym.non_terminal() {
                    Some(name) => {
                        graph.add_edge(nodes[rule.lhs.as_str()], nodes[name], ());
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

    pub fn check_duplicate(&self) -> crate::error::Result<&Self> {
        // TODO: need to rework
        Ok(self)
    }

    pub fn check_repeats(&self) -> crate::error::Result<&Self> {
        for rule in &self.rules {
            for alt in rule.rhs() {
                if let Limit::Limited { min, max } = alt.invoke_limit {
                    if min > max {
                        return Err(Error::InvalidRepeatRange { span: alt.span });
                    }
                }
            }
        }
        Ok(self)
    }

    pub fn check_undefined(&self) -> crate::error::Result<&Self> {
        let defined: HashSet<String> =
            HashSet::from_iter(self.rules.iter().map(|r| r.lhs.as_str().to_string()));
        for rule in &self.rules {
            for sym in rule.rhs().iter().flat_map(|a| a.symbols.iter()) {
                match &sym.kind {
                    SymbolKind::NonTerminal(s) => {
                        if !defined.contains(s.as_str()) {
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
