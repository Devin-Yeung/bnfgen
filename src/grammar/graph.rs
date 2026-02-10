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
        // Convert SCC node indices to a set of rule names for quick lookup
        let scc_names: HashSet<&str> = scc.iter().map(|nx| self.graph[*nx].as_str()).collect();

        // Get the rules in this SCC
        let scc_rules: Vec<&Rule> = self
            .rules
            .iter()
            .filter(|r| scc_names.contains(r.lhs.as_str()))
            .collect();

        // Use fixpoint computation to find which rules in the SCC can escape
        // A rule can escape if it has an alternative that can escape
        // An alternative can escape if its first symbol can escape
        // A terminal/regex symbol can always escape
        // A non-terminal symbol can escape if it's NOT in the SCC, or if it can escape
        let mut can_escape: HashSet<&str> = HashSet::new();

        // Iteratively find rules that can escape until no more changes
        loop {
            let prev_len = can_escape.len();

            for rule in &scc_rules {
                if can_escape.contains(rule.lhs.as_str()) {
                    continue; // Already known to be escapable
                }

                // Check if any alternative can escape
                let rule_can_escape = rule.rhs().iter().any(|alt| {
                    // An alternative can escape if all its symbols can escape
                    // (for generation to make progress through this alternative)
                    alt.symbols.iter().all(|sym| {
                        if sym.kind.is_terminal() {
                            // Terminals can always escape
                            true
                        } else if let Some(nt_name) = sym.kind.non_terminal() {
                            // Non-terminal: can escape if not in SCC or known to escape
                            !scc_names.contains(nt_name) || can_escape.contains(nt_name)
                        } else {
                            false
                        }
                    })
                });

                if rule_can_escape {
                    can_escape.insert(rule.lhs.as_str());
                }
            }

            // If no new rules were found to escape, we're done
            if can_escape.len() == prev_len {
                break;
            }
        }

        // It's a trap loop if NO rule in the SCC can escape
        !scc_rules.is_empty() && can_escape.is_empty()
    }
}
