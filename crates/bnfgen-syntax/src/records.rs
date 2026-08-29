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

/// One recognized rule: `<name> ::= … ;` or `<name: "type"> ::= … ;`.
#[derive(Debug)]
pub(crate) struct RuleRecord {
    /// The whole rule text, from `<` through `;`.
    pub(crate) span: Range<usize>,
    /// The bracketed left-hand-side form. The same record shape represents
    /// declarations and references, so views never reconstruct typed syntax
    /// by scanning tokens.
    pub(crate) lhs: NonTerminalRecord,
    /// The `|`-separated alternatives.
    pub(crate) alts: Vec<AlternativeRecord>,
}

/// One alternative on a rule's right-hand side.
#[derive(Debug)]
pub(crate) struct AlternativeRecord {
    /// The whole alternative, including an optional weight and repeat.
    pub(crate) span: Range<usize>,
    /// Raw integer token before the first symbol, when present.
    pub(crate) weight: Option<Range<usize>>,
    pub(crate) symbols: Vec<SymbolRecord>,
    /// The trailing invocation-limit clause, when present.
    pub(crate) repeat: Option<RepeatRecord>,
}

/// One `<name>` or `<name: "type">` occurrence.
#[derive(Debug)]
pub(crate) struct NonTerminalRecord {
    pub(crate) span: Range<usize>,
    pub(crate) name: Range<usize>,
    /// The raw quoted type lexeme. Decoding belongs to analysis.
    pub(crate) ty: Option<Range<usize>>,
}

/// A trailing invocation-limit clause.
///
/// These fields describe spelling, not meaning. In particular, `{5}` and
/// `{5,}` both have no explicit upper token; `comma` distinguishes them so
/// downstream lowering can apply the legacy semantics without guessing.
#[derive(Debug)]
pub(crate) struct RepeatRecord {
    pub(crate) span: Range<usize>,
    pub(crate) lower: Range<usize>,
    pub(crate) comma: Option<Range<usize>>,
    pub(crate) upper: Option<Range<usize>>,
}

/// One symbol within an alternative.
#[derive(Debug)]
pub(crate) enum SymbolRecord {
    /// A string literal, kept raw (quotes included, escapes undecoded).
    Terminal { span: Range<usize> },
    /// A typed or untyped non-terminal reference.
    NonTerminal(NonTerminalRecord),
    /// `re("pattern")`, kept raw and uncompiled.
    Regex {
        span: Range<usize>,
        pattern: Range<usize>,
    },
}
