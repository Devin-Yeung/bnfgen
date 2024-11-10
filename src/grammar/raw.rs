use crate::error::Error;
use crate::grammar::alt::Limit;
use crate::grammar::checked::CheckedGrammar;
use crate::grammar::graph::GrammarGraph;
use crate::grammar::rule::Rule;
use crate::grammar::symbol::SymbolKind;
use crate::lexer;
use crate::span::Span;
use crate::utils::convert_parse_error;
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

    pub fn check_duplicate(&self) -> crate::error::Result<&Self> {
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
            HashSet::from_iter(self.rules.iter().map(|r| r.name.clone()));
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
