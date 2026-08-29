//! Private, byte-range-backed storage for recognized grammar structure.
//!
//! These records are deliberately *not* a tree: each one names UTF-8 byte
//! ranges into the retained source, so recognized syntax never copies text
//! and never needs trivia reinserted. They are an implementation detail —
//! the public surface wraps them in the borrowing views in [`crate::views`],
//! and storage may change without breaking callers.
//!
//! A record describes only syntax that was actually observed. Required
//! children are therefore optional: recovery may know that a rule or nested
//! form has begun without inventing the delimiter or value that would finish
//! it. No missing child is represented by a zero-width range.

use std::ops::Range;

/// One recognized rule: `<name> ::= … ;` or `<name: "type"> ::= … ;`.
#[derive(Debug)]
pub(crate) struct RuleRecord {
    /// The observed rule text. A complete rule runs from `<` through `;`.
    pub(crate) span: Range<usize>,
    /// The bracketed left-hand-side form. The same record shape represents
    /// declarations and references, so views never reconstruct typed syntax
    /// by scanning tokens.
    pub(crate) lhs: Option<NonTerminalRecord>,
    /// The `::=` token, when written.
    pub(crate) definition: Option<Range<usize>>,
    /// The `|`-separated alternatives.
    pub(crate) alts: Vec<AlternativeRecord>,
    /// The terminating `;`, when written.
    pub(crate) terminator: Option<Range<usize>>,
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
    pub(crate) opening: Range<usize>,
    pub(crate) name: Option<Range<usize>>,
    pub(crate) type_separator: Option<Range<usize>>,
    /// The raw quoted type lexeme. Decoding belongs to analysis.
    pub(crate) ty: Option<StringRecord>,
    pub(crate) closing: Option<Range<usize>>,
}

/// A complete or unterminated quoted string.
///
/// The lexer already decides whether the closing quote exists. Keeping that
/// fact beside the range lets terminals, types, and regex patterns share one
/// truthful representation without decoding their contents.
#[derive(Debug)]
pub(crate) struct StringRecord {
    pub(crate) span: Range<usize>,
    pub(crate) terminated: bool,
}

/// A trailing invocation-limit clause.
///
/// These fields describe spelling, not meaning. In particular, `{5}` and
/// `{5,}` both have no explicit upper token; `comma` distinguishes them so
/// downstream lowering can apply the legacy semantics without guessing.
#[derive(Debug)]
pub(crate) struct RepeatRecord {
    pub(crate) span: Range<usize>,
    pub(crate) opening: Range<usize>,
    pub(crate) lower: Option<Range<usize>>,
    pub(crate) comma: Option<Range<usize>>,
    pub(crate) upper: Option<Range<usize>>,
    pub(crate) closing: Option<Range<usize>>,
}

/// A complete or partially typed `re("pattern")` symbol.
#[derive(Debug)]
pub(crate) struct RegexRecord {
    pub(crate) span: Range<usize>,
    pub(crate) opening_parenthesis: Option<Range<usize>>,
    pub(crate) pattern: Option<StringRecord>,
    pub(crate) closing_parenthesis: Option<Range<usize>>,
}

/// One symbol within an alternative.
#[derive(Debug)]
pub(crate) enum SymbolRecord {
    /// A string literal, kept raw (quotes included, escapes undecoded).
    Terminal(StringRecord),
    /// A typed or untyped non-terminal reference.
    NonTerminal(NonTerminalRecord),
    /// `re("pattern")`, kept raw and uncompiled.
    Regex(RegexRecord),
}
