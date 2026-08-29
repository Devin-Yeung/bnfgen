//! Language-specific immutable views over a parsed document.
//!
//! Views borrow the document and wrap its private records, so callers can
//! never retain syntax independently of the source it came from.
//! Every accessor returns either raw text sliced from the document or a
//! UTF-8 byte range — never a decoded or validated value. Accessors for
//! required children are `Option` because recovery can publish a rule-shaped
//! prefix without inventing the syntax that would complete it.

use std::ops::Range;

use crate::document::ParsedDocument;
use crate::records::{
    AlternativeRecord, NonTerminalRecord, RegexRecord, RepeatRecord, RuleRecord, StringRecord,
    SymbolRecord,
};

/// A recognized rule: its left-hand side, alternatives, and source ranges.
#[derive(Debug, Clone)]
pub struct RuleSyntax<'a> {
    doc: &'a ParsedDocument,
    record: &'a RuleRecord,
}

impl<'a> RuleSyntax<'a> {
    pub(super) fn new(doc: &'a ParsedDocument, record: &'a RuleRecord) -> Self {
        Self { doc, record }
    }

    /// The rule's full source range, from `<` through `;`.
    pub fn range(&self) -> Range<usize> {
        self.record.span.clone()
    }

    /// The rule's raw text, exactly as written.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.record.span.clone())
    }

    /// The left-hand-side non-terminal, when enough of one was recognized.
    pub fn lhs(&self) -> Option<NonTerminalSyntax<'a>> {
        self.record
            .lhs
            .as_ref()
            .map(|record| NonTerminalSyntax::new(self.doc, record))
    }

    /// The written `::=` token, absent while a declaration is incomplete.
    pub fn definition_range(&self) -> Option<Range<usize>> {
        self.record.definition.clone()
    }

    /// The `|`-separated alternatives, in source order.
    pub fn alternatives(&self) -> impl Iterator<Item = AlternativeSyntax<'a>> + 'a {
        let doc = self.doc;
        self.record
            .alts
            .iter()
            .map(move |record| AlternativeSyntax::new(doc, record))
    }

    /// The written `;`, absent while a rule is incomplete.
    pub fn terminator_range(&self) -> Option<Range<usize>> {
        self.record.terminator.clone()
    }
}

/// A non-terminal occurrence: a rule's left-hand side or a `<name>`
/// reference on a right-hand side.
///
/// The view distinguishes the whole bracketed form (`.range()`/`.text()`,
/// delimiters included) from the bare identifier (`.name_range()`/`.name()`).
#[derive(Debug, Clone)]
pub struct NonTerminalSyntax<'a> {
    doc: &'a ParsedDocument,
    record: &'a NonTerminalRecord,
}

impl<'a> NonTerminalSyntax<'a> {
    pub(super) fn new(doc: &'a ParsedDocument, record: &'a NonTerminalRecord) -> Self {
        Self { doc, record }
    }

    /// The full bracketed form's source range.
    pub fn range(&self) -> Range<usize> {
        self.record.span.clone()
    }

    /// The full bracketed form's raw text, delimiters included.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.record.span.clone())
    }

    /// The opening `<` that establishes this non-terminal-shaped fact.
    pub fn opening_delimiter_range(&self) -> Range<usize> {
        self.record.opening.clone()
    }

    /// The bare identifier's source range.
    pub fn name_range(&self) -> Option<Range<usize>> {
        self.record.name.clone()
    }

    /// The identifier's raw spelling, absent for a bare opening `<`.
    pub fn name(&self) -> Option<&'a str> {
        self.record
            .name
            .as_ref()
            .map(|range| self.doc.slice(range.clone()))
    }

    /// The written `:` before a type annotation.
    ///
    /// This distinguishes an untyped occurrence from one whose type string
    /// has not been written yet.
    pub fn type_separator_range(&self) -> Option<Range<usize>> {
        self.record.type_separator.clone()
    }

    /// The quoted type annotation when this occurrence is typed.
    ///
    /// The returned literal includes its quotes and escapes. Syntax does not
    /// decode it or decide what type identity it denotes.
    pub fn ty(&self) -> Option<StringSyntax<'a>> {
        self.record
            .ty
            .as_ref()
            .map(|record| StringSyntax::new(self.doc, record))
    }

    /// The written `>`, absent while the occurrence is incomplete.
    pub fn closing_delimiter_range(&self) -> Option<Range<usize>> {
        self.record.closing.clone()
    }
}

/// One alternative of a rule's right-hand side.
#[derive(Debug, Clone)]
pub struct AlternativeSyntax<'a> {
    doc: &'a ParsedDocument,
    record: &'a AlternativeRecord,
}

impl<'a> AlternativeSyntax<'a> {
    pub(super) fn new(doc: &'a ParsedDocument, record: &'a AlternativeRecord) -> Self {
        Self { doc, record }
    }

    /// The alternative's source range, from its first to its last symbol.
    pub fn range(&self) -> Range<usize> {
        self.record.span.clone()
    }

    /// The alternative's raw text.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.record.span.clone())
    }

    /// The optional raw integer weight before the first symbol.
    pub fn weight(&self) -> Option<IntegerSyntax<'a>> {
        self.record
            .weight
            .as_ref()
            .map(|span| IntegerSyntax::new(self.doc, span.clone()))
    }

    /// The alternative's symbols, in source order.
    pub fn symbols(&self) -> impl Iterator<Item = SymbolSyntax<'a>> + 'a {
        let doc = self.doc;
        self.record
            .symbols
            .iter()
            .map(move |record| SymbolSyntax::new(doc, record))
    }

    /// The optional trailing invocation-limit clause.
    pub fn repeat(&self) -> Option<RepeatSyntax<'a>> {
        self.record
            .repeat
            .as_ref()
            .map(|record| RepeatSyntax::new(self.doc, record))
    }
}

/// The form a symbol takes. Payloads are reached through the typed view
/// methods on [`SymbolSyntax`] rather than carried here, keeping the kind
/// cheap to match on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Terminal,
    NonTerminal,
    Regex,
}

/// One symbol within an alternative.
#[derive(Debug, Clone)]
pub struct SymbolSyntax<'a> {
    doc: &'a ParsedDocument,
    record: &'a SymbolRecord,
}

impl<'a> SymbolSyntax<'a> {
    pub(super) fn new(doc: &'a ParsedDocument, record: &'a SymbolRecord) -> Self {
        Self { doc, record }
    }

    /// The symbol's source range.
    pub fn range(&self) -> Range<usize> {
        match self.record {
            SymbolRecord::Terminal(record) => record.span.clone(),
            SymbolRecord::NonTerminal(record) => record.span.clone(),
            SymbolRecord::Regex(record) => record.span.clone(),
        }
    }

    /// The symbol's raw text — for a terminal, the literal including
    /// quotes and undecoded escapes.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.range())
    }

    pub fn kind(&self) -> SymbolKind {
        match self.record {
            SymbolRecord::Terminal(_) => SymbolKind::Terminal,
            SymbolRecord::NonTerminal(_) => SymbolKind::NonTerminal,
            SymbolRecord::Regex(_) => SymbolKind::Regex,
        }
    }

    /// The raw quoted literal when this symbol is a terminal.
    pub fn as_terminal(&self) -> Option<StringSyntax<'a>> {
        match self.record {
            SymbolRecord::Terminal(record) => Some(StringSyntax::new(self.doc, record)),
            SymbolRecord::NonTerminal(_) | SymbolRecord::Regex(_) => None,
        }
    }

    /// The non-terminal view when this symbol is a non-terminal reference.
    pub fn as_non_terminal(&self) -> Option<NonTerminalSyntax<'a>> {
        match self.record {
            SymbolRecord::NonTerminal(record) => Some(NonTerminalSyntax::new(self.doc, record)),
            SymbolRecord::Terminal(_) | SymbolRecord::Regex(_) => None,
        }
    }

    /// The regular-expression form when this symbol is `re("pattern")`.
    pub fn as_regex(&self) -> Option<RegexSyntax<'a>> {
        match self.record {
            SymbolRecord::Regex(record) => Some(RegexSyntax::new(self.doc, record)),
            SymbolRecord::Terminal(_) | SymbolRecord::NonTerminal(_) => None,
        }
    }
}

/// Raw quoted string syntax used as a terminal, type, or regex pattern.
///
/// The view also represents a string whose closing quote has not been typed;
/// callers get the observed text and can test termination without decoding it.
#[derive(Debug, Clone)]
pub struct StringSyntax<'a> {
    doc: &'a ParsedDocument,
    record: &'a StringRecord,
}

impl<'a> StringSyntax<'a> {
    fn new(doc: &'a ParsedDocument, record: &'a StringRecord) -> Self {
        Self { doc, record }
    }

    pub fn range(&self) -> Range<usize> {
        self.record.span.clone()
    }

    /// Exact source spelling, including quotes and undecoded escapes.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.record.span.clone())
    }

    /// Whether the lexer observed a closing quote.
    pub fn is_terminated(&self) -> bool {
        self.record.terminated
    }
}

/// A raw integer token used as an alternative weight or repeat bound.
#[derive(Debug, Clone)]
pub struct IntegerSyntax<'a> {
    doc: &'a ParsedDocument,
    span: Range<usize>,
}

impl<'a> IntegerSyntax<'a> {
    fn new(doc: &'a ParsedDocument, span: Range<usize>) -> Self {
        Self { doc, span }
    }

    pub fn range(&self) -> Range<usize> {
        self.span.clone()
    }

    /// Exact decimal spelling. Parsing and overflow checks are downstream.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.span.clone())
    }
}

/// Raw syntax for `{n}`, `{n,}`, or `{n, m}`.
#[derive(Debug, Clone)]
pub struct RepeatSyntax<'a> {
    doc: &'a ParsedDocument,
    record: &'a RepeatRecord,
}

impl<'a> RepeatSyntax<'a> {
    fn new(doc: &'a ParsedDocument, record: &'a RepeatRecord) -> Self {
        Self { doc, record }
    }

    pub fn range(&self) -> Range<usize> {
        self.record.span.clone()
    }

    pub fn text(&self) -> &'a str {
        self.doc.slice(self.record.span.clone())
    }

    /// The opening `{` that establishes this repeat-shaped fact.
    pub fn opening_delimiter_range(&self) -> Range<usize> {
        self.record.opening.clone()
    }

    pub fn lower_bound(&self) -> Option<IntegerSyntax<'a>> {
        self.record
            .lower
            .as_ref()
            .map(|span| IntegerSyntax::new(self.doc, span.clone()))
    }

    /// The comma distinguishes exact `{n}` from open `{n,}` syntax.
    pub fn comma_range(&self) -> Option<Range<usize>> {
        self.record.comma.clone()
    }

    /// The explicitly written upper bound, absent in `{n}` and `{n,}`.
    pub fn upper_bound(&self) -> Option<IntegerSyntax<'a>> {
        self.record
            .upper
            .as_ref()
            .map(|span| IntegerSyntax::new(self.doc, span.clone()))
    }

    /// The written `}`, absent while the repeat is incomplete.
    pub fn closing_delimiter_range(&self) -> Option<Range<usize>> {
        self.record.closing.clone()
    }
}

/// Raw syntax for a regular-expression symbol, `re("pattern")`.
#[derive(Debug, Clone)]
pub struct RegexSyntax<'a> {
    doc: &'a ParsedDocument,
    record: &'a RegexRecord,
}

impl<'a> RegexSyntax<'a> {
    fn new(doc: &'a ParsedDocument, record: &'a RegexRecord) -> Self {
        Self { doc, record }
    }

    pub fn range(&self) -> Range<usize> {
        self.record.span.clone()
    }

    pub fn text(&self) -> &'a str {
        self.doc.slice(self.record.span.clone())
    }

    /// The written `(`, absent after a bare `re` prefix.
    pub fn opening_parenthesis_range(&self) -> Option<Range<usize>> {
        self.record.opening_parenthesis.clone()
    }

    /// The raw, uncompiled regex pattern when one has begun.
    pub fn pattern(&self) -> Option<StringSyntax<'a>> {
        self.record
            .pattern
            .as_ref()
            .map(|record| StringSyntax::new(self.doc, record))
    }

    /// The written `)`, absent while the regex form is incomplete.
    pub fn closing_parenthesis_range(&self) -> Option<Range<usize>> {
        self.record.closing_parenthesis.clone()
    }
}
