//! Private, byte-range-backed storage for recognized grammar structure.
//!
//! Per ADR 0004 these records are deliberately *not* a tree: each one names
//! UTF-8 byte ranges into the retained source, so recognized syntax never
//! copies text and never needs trivia reinserted. They are an implementation
//! detail — the public surface wraps them in the borrowing views in
//! [`crate::views`], and storage may change without breaking callers.
//!
//! Children are non-optional in this slice because a record only exists once
//! the grammar has fully recognized it. Recovery (tickets 04/05) will admit
//! partially recognized rules; their records will hold `Option` children and
//! the views will surface those as `None`.

use std::ops::Range;

/// One recognized rule: `<name> ::= … ;`
#[derive(Debug)]
pub(crate) struct RuleRecord {
    /// The whole rule text, from `<` through `;`.
    pub(crate) span: Range<usize>,
    /// The bracketed left-hand-side form, `<name>` — today the rule's
    /// opening `<` through its closing `>`.
    pub(crate) lhs: Range<usize>,
    /// The identifier inside the left-hand-side angle brackets.
    pub(crate) name: Range<usize>,
    /// The `|`-separated alternatives.
    pub(crate) alts: Vec<AlternativeRecord>,
}

/// One alternative on a rule's right-hand side.
#[derive(Debug)]
pub(crate) struct AlternativeRecord {
    /// From the first symbol to the last symbol of this alternative.
    pub(crate) span: Range<usize>,
    pub(crate) symbols: Vec<SymbolRecord>,
}

/// One symbol within an alternative.
#[derive(Debug)]
pub(crate) enum SymbolRecord {
    /// A string literal, kept raw (quotes included, escapes undecoded).
    Terminal { span: Range<usize> },
    /// A non-terminal reference `<name>`.
    NonTerminal {
        span: Range<usize>,
        name: Range<usize>,
    },
}
