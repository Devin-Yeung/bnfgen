//! Grammar graph for static analysis.
//!
//! This module provides [`GrammarGraph`] for analyzing grammar structure
//! using graph algorithms. It can detect unreachable rules and trap loops
//! (cycles that cannot produce terminals).

use crate::error::Error;
use crate::grammar::rule::Rule;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::prelude::Dfs;
use std::collections::{HashMap, HashSet};

/// Graph representation of a BNF grammar for static analysis.
///
/// `GrammarGraph` uses [petgraph] to represent the grammar as a directed
/// graph where nodes are non-terminals and edges represent references
/// between them. This enables powerful static analysis like detecting
/// unreachable rules and trap loops.
///
/// # Example
///
/// ```rust
/// use bnfgen::RawGrammar;
///
/// let grammar = RawGrammar::parse(
///     r#"
///     <S> ::= "a" | <B> ;
///     <B> ::= "b" ;
///     <C> ::= "c" ;
///     "#
/// ).unwrap();
///
/// let graph = grammar.graph();
/// // Check for unreachable rules from "S"
/// match graph.check_unused("S") {
///     Ok(_) => println!("All rules reachable"),
///     Err(e) => eprintln!("Found unreachable rules: {}", e),
/// }
/// ```
///
/// [petgraph]: https://docs.rs/petgraph
pub struct GrammarGraph<'rule> {
    pub(crate) rules: &'rule Vec<Rule>,
    pub(crate) graph: DiGraph<String, ()>,
    pub(crate) nodes: HashMap<String, NodeIndex>,
}

impl<'rule> GrammarGraph<'rule> {
    /// Checks for unreachable rules from a given start symbol.
    ///
    /// Uses depth-first search to find all rules reachable from the start
    /// symbol, then reports any rules that cannot be reached.
    ///
    /// # Arguments
    ///
    /// * `start` - The name of the start non-terminal (e.g., "S")
    ///
    /// # Errors
    ///
    /// Returns `Error::UnreachableRules` if any rules are unreachable from
    /// the start symbol.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::RawGrammar;
    ///
    /// let grammar = RawGrammar::parse(
    ///     r#"
    ///     <S> ::= "a" ;
    ///     <Unused> ::= "b" ;
    ///     "#
    /// ).unwrap();
    ///
    /// let graph = grammar.graph();
    /// assert!(graph.check_unused("S").is_err());
    /// ```
    pub fn check_unused<S: AsRef<str>>(&self, start: S) -> crate::error::Result<&Self> {
        let all_nts = self
            .nodes
            .keys()
            .map(|s| s.as_str())
            .collect::<HashSet<_>>();
        // find the reachable nodes for a given start symbol
        let start = self
            .nodes
            .get(start.as_ref())
            .expect("The start symbol does not exist");

        let mut dfs = Dfs::new(&self.graph, *start);
        let mut reachable = HashSet::new();
        while let Some(nx) = dfs.next(&self.graph) {
            let name = &self.graph[nx];
            reachable.insert(name.as_str());
        }
        let unreachable = all_nts.difference(&reachable).collect::<HashSet<_>>();
        // find the unreachable spans
        if !unreachable.is_empty() {
            let spans = self
                .rules
                .iter()
                .filter(|rule| unreachable.contains(&&rule.lhs.as_str()))
                .map(|rule| rule.span)
                .collect::<Vec<_>>();
            return Err(Error::UnreachableRules { spans });
        }
        Ok(self)
    }

    /// Checks for trap loops (dead loops) in the grammar.
    ///
    /// A trap loop is a strongly connected component (SCC) where:
    /// 1. No rule in the SCC can produce a terminal
    /// 2. All outgoing edges from the SCC stay within the SCC
    ///
    /// Such loops are "traps" because generation can never escape them.
    ///
    /// # Errors
    ///
    /// Returns `Error::TrapLoop` if any trap loops are detected.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::RawGrammar;
    ///
    /// let grammar = RawGrammar::parse(
    ///     r#"
    ///     <A> ::= <B> ;
    ///     <B> ::= <A> ;
    ///     "#
    /// ).unwrap();
    ///
    /// let graph = grammar.graph();
    /// assert!(graph.check_trap_loop().is_err());
    /// ```
    pub fn check_trap_loop(&self) -> crate::error::Result<&Self> {
        let sccs = petgraph::algo::tarjan_scc(&self.graph);
        for scc in sccs {
            if self.is_trap_loop(&scc) {
                let spans = scc
                    .iter()
                    .map(|nx| {
                        self.rules
                            .iter()
                            .find(|rule| rule.lhs.as_str() == self.graph[*nx])
                            .unwrap()
                            .span
                    })
                    .collect::<Vec<_>>();
                return Err(Error::TrapLoop { spans });
            }
        }
        Ok(self)
    }

    fn is_trap_loop(&self, scc: &[NodeIndex]) -> bool {
        let produce_t = scc.iter().map(|nx| self.graph[*nx].as_str()).any(|name| {
            // check if rule produce a terminal
            self.rules
                .iter()
                .find(|rule| rule.lhs.as_str() == name)
                .unwrap()
                .produce_terminals()
        });
        if produce_t {
            return false;
        }
        let out_deg: HashSet<NodeIndex> = scc
            .iter()
            .flat_map(|nx| self.graph.neighbors(*nx))
            .collect();
        out_deg == scc.iter().copied().collect()
    }
}
