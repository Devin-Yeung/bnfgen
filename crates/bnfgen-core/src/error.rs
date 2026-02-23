//! Error types for bnfgen.
//!
//! This module defines the [`Error`] enum representing all possible errors
//! that can occur when parsing and validating BNF grammars. All errors
//! implement [`miette::Diagnostic`] for rich error reporting with source
//! code annotations.

use crate::span::Span;

/// Result type for bnfgen operations.
///
/// `Result<T>` is an alias for `std::result::Result<T, Error>`.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors that can occur when parsing and validating BNF grammars.
///
/// All error variants implement [`miette::Diagnostic`] for rich error
/// messages with source code annotations and helpful labels.
///
/// # Example
///
/// ```rust
/// use bnfgen::{RawGrammar, Error};
///
/// match RawGrammar::parse("<S> ::= <Undefined>;") {
///     Ok(_) => println!("Valid"),
///     Err(Error::UndefinedNonTerminal { .. }) => {
///         println!("Undefined non-terminal referenced");
///     }
///     Err(e) => println!("Other error: {}", e),
/// }
/// ```
#[derive(thiserror::Error, miette::Diagnostic, Debug, Eq, PartialEq, Clone)]
pub enum Error {
    /// Unrecognized token during parsing.
    ///
    /// This error occurs when the parser encounters a token that doesn't
    /// match the expected grammar syntax.
    #[error("Unrecognized token")]
    UnrecognizedToken {
        /// The location of the unrecognized token
        #[label("expect {expect}")]
        span: Span,
        /// Description of what was expected
        expect: String,
    },

    /// Extra token after grammar was fully parsed.
    ///
    /// This occurs when there's additional content after the grammar
    /// is complete (e.g., trailing characters).
    #[error("Unexpected extra token")]
    ExtraToken {
        /// The location of the extra token
        #[label("this extra token is unexpected")]
        span: Span,
    },

    /// Unexpected end of input during parsing.
    ///
    /// This occurs when the input ends before the grammar is complete.
    #[error("Unrecognized EOF")]
    UnrecognizedEof {
        /// The location where EOF was encountered
        #[label("expect {expect}")]
        span: Span,
        /// Description of what was expected
        expect: String,
    },

    /// Reference to an undefined non-terminal.
    ///
    /// This error occurs when a non-terminal is referenced in a rule
    /// but never defined.
    #[error("Undefined non-terminal")]
    UndefinedNonTerminal {
        /// The location of the undefined non-terminal reference
        #[label("this non-terminal is undefined")]
        span: Span,
    },

    /// Duplicate rule definitions detected.
    ///
    /// This error occurs when the same non-terminal is defined multiple
    /// times (currently unused - see [`crate::grammar::raw::RawGrammar::check_duplicate()`]).
    #[error("Duplicated rules found")]
    DuplicatedRules {
        /// The location of the duplicate definition
        #[label("this rule is duplicated")]
        span: Span,
        /// The location of the original definition
        #[label("previous defined here")]
        prev: Span,
    },

    /// Invalid invoke limit range.
    ///
    /// This error occurs when an alternative has a `{min, max}` invoke
    /// limit where `min > max`.
    #[error("Invalid repeat range")]
    InvalidRepeatRange {
        /// The location of the invalid range
        #[label("min should be less than or equal to max")]
        span: Span,
    },

    /// Unreachable rules detected.
    ///
    /// This error occurs when [`crate::grammar::graph::GrammarGraph::check_unused()`] finds rules
    /// that cannot be reached from the specified start symbol.
    #[error("Found unreachable rules")]
    UnreachableRules {
        /// The locations of all unreachable rules
        #[label(collection, "this rule is unreachable")]
        spans: Vec<Span>,
    },

    /// Potential trap loop (dead loop) detected.
    ///
    /// This error occurs when [`crate::grammar::graph::GrammarGraph::check_trap_loop()`] finds
    /// a cycle in the grammar that cannot produce terminals, meaning
    /// generation would never terminate.
    #[error("May be trapped in a dead loop")]
    TrapLoop {
        /// The locations of all rules in the trap loop
        #[label(collection, "this rule may be trapped in a dead loop")]
        spans: Vec<Span>,
    },

    /// Invalid regular expression.
    ///
    /// This error occurs when a `re("...")` pattern contains an invalid
    /// regex syntax.
    #[error("Invalid regex")]
    InvalidRegex {
        /// The location of the invalid regex
        #[label("this regex is invalid")]
        span: Span,
    },

    /// Lexical error from tokenization.
    ///
    /// This wraps errors that occur during the lexing phase.
    #[error(transparent)]
    #[diagnostic(transparent)]
    LexicalError(#[from] crate::token::LexicalError),

    /// No valid candidates available for expansion.
    ///
    /// This error occurs during generation when all alternatives for a
    /// non-terminal have exceeded their invoke limits, leaving no valid
    /// options to continue generation.
    #[error("No candidates available for non-terminal `{name}`")]
    NoCandidatesAvailable {
        /// The name of the non-terminal that has no available candidates
        name: String,
    },
}
