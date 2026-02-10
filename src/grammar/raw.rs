//! Raw grammar parsing module.
//!
//! This module provides [`RawGrammar`] for parsing BNF grammar text.
//! After parsing, use [`RawGrammar::to_checked()`] to validate and convert
//! to a [`CheckedGrammar`] suitable for generation.

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

/// Raw parsed BNF grammar before validation.
///
/// `RawGrammar` is the result of parsing BNF grammar text. It contains
/// the parsed rules but has not yet been validated. Before using for
/// generation, call [`to_checked()`](Self::to_checked) to validate and
/// convert to a [`CheckedGrammar`].
///
/// # Example
///
/// ```rust
/// use bnfgen::RawGrammar;
///
/// let grammar = RawGrammar::parse(
///     r#"
///     <S> ::= "hello" | "world" ;
///     "#
/// ).unwrap();
///
/// // Validate and convert
/// let checked = grammar.to_checked().unwrap();
/// ```
#[repr(transparent)]
#[derive(Debug)]
pub struct RawGrammar {
    pub(crate) rules: Vec<Rule>,
}

impl RawGrammar {
    /// Parses a BNF grammar from a string.
    ///
    /// # Arguments
    ///
    /// * `input` - A string or string slice containing the BNF grammar
    ///
    /// # Returns
    ///
    /// A `Result` containing the parsed `RawGrammar` or an `Error` if
    /// parsing fails (e.g., syntax errors, invalid regex).
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::RawGrammar;
    ///
    /// let grammar = RawGrammar::parse("<S> ::= \"a\" | \"b\";").unwrap();
    /// ```
    pub fn parse<S: AsRef<str>>(input: S) -> crate::error::Result<RawGrammar> {
        let lexer = lexer::Lexer::new(input.as_ref());
        let parser = crate::parser::RawGrammarParser::new();
        parser.parse(lexer).map_err(convert_parse_error)
    }

    /// Validates this grammar and converts it to a `CheckedGrammar`.
    ///
    /// This runs all validation checks:
    ///
    /// - [`check_undefined()`](Self::check_undefined) - Ensures all referenced
    ///   non-terminals are defined
    /// - [`check_duplicate()`](Self::check_duplicate) - Checks for duplicate
    ///   rule definitions (currently disabled)
    /// - [`check_repeats()`](Self::check_repeats) - Validates invoke limits
    ///   (min <= max)
    ///
    /// # Errors
    ///
    /// Returns an `Error` if any validation check fails.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::RawGrammar;
    ///
    /// let grammar = RawGrammar::parse("<S> ::= \"a\" | \"b\";").unwrap();
    /// let checked = grammar.to_checked().unwrap();
    /// ```
    pub fn to_checked(self) -> crate::error::Result<CheckedGrammar> {
        self.check_undefined()?.check_duplicate()?.check_repeats()?;

        let mut rules = IndexMap::new();
        for rule in self.rules {
            rules.insert(rule.lhs, rule.production);
        }

        Ok(CheckedGrammar { rules })
    }

    /// Creates a graph representation for static analysis.
    ///
    /// Returns a [`GrammarGraph`] that can be used for advanced analysis
    /// like detecting unreachable rules and trap loops.
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
    /// if let Err(e) = graph.check_unused("S") {
    ///     eprintln!("Unreachable rules found: {:?}", e);
    /// }
    /// ```
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

    /// Checks for duplicate rule definitions.
    ///
    /// **Note**: This check is currently unimplemented and always returns `Ok(())`.
    ///
    /// # TODO
    ///
    /// This check needs to be reworked to properly detect duplicate definitions
    /// of the same non-terminal (including typed non-terminals).
    pub fn check_duplicate(&self) -> crate::error::Result<&Self> {
        // TODO: need to rework
        Ok(self)
    }

    /// Validates that all invoke limits have valid ranges.
    ///
    /// Ensures that for any alternative with a `{min, max}` invoke limit,
    /// the minimum value is less than or equal to the maximum value.
    ///
    /// # Errors
    ///
    /// Returns `Error::InvalidRepeatRange` if any invoke limit has min > max.
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

    /// Checks that all referenced non-terminals are defined.
    ///
    /// Ensures that every non-terminal referenced in the right-hand side
    /// of rules has a corresponding definition.
    ///
    /// # Errors
    ///
    /// Returns `Error::UndefinedNonTerminal` if any undefined non-terminal
    /// is referenced.
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
