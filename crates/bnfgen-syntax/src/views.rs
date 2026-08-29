//! Language-specific immutable views over a parsed document.
//!
//! Views borrow the document and wrap its private records, so callers can
//! never retain syntax independently of the source it came from.
//! Every accessor returns either raw text sliced from the document or a
//! UTF-8 byte range — never a decoded or validated value. Accessors for
//! required children become `Option` once recovery can produce partially
//! recognized rules; today a view exists only for fully recognized syntax.

use std::ops::Range;

use crate::document::ParsedDocument;
use crate::records::{
    AlternativeRecord, NonTerminalRecord, RepeatRecord, RuleRecord, SymbolRecord,
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

    /// The left-hand-side non-terminal, e.g. `<greeting>`.
    pub fn name(&self) -> NonTerminalSyntax<'a> {
        NonTerminalSyntax::new(self.doc, &self.record.lhs)
    }

    /// The `|`-separated alternatives, in source order.
    pub fn alternatives(&self) -> impl Iterator<Item = AlternativeSyntax<'a>> + 'a {
        let doc = self.doc;
        self.record
            .alts
            .iter()
            .map(move |record| AlternativeSyntax::new(doc, record))
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

    /// The bare identifier's source range.
    pub fn name_range(&self) -> Range<usize> {
        self.record.name.clone()
    }

    /// The identifier's raw spelling.
    pub fn name(&self) -> &'a str {
        self.doc.slice(self.record.name.clone())
    }

    /// The quoted type annotation when this occurrence is typed.
    ///
    /// The returned literal includes its quotes and escapes. Syntax does not
    /// decode it or decide what type identity it denotes.
    pub fn ty(&self) -> Option<StringLiteralSyntax<'a>> {
        self.record
            .ty
            .as_ref()
            .map(|span| StringLiteralSyntax::new(self.doc, span.clone()))
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
            SymbolRecord::Terminal { span } => span.clone(),
            SymbolRecord::NonTerminal(record) => record.span.clone(),
            SymbolRecord::Regex { span, .. } => span.clone(),
        }
    }

    /// The symbol's raw text — for a terminal, the literal including
    /// quotes and undecoded escapes.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.range())
    }

    pub fn kind(&self) -> SymbolKind {
        match self.record {
            SymbolRecord::Terminal { .. } => SymbolKind::Terminal,
            SymbolRecord::NonTerminal(_) => SymbolKind::NonTerminal,
            SymbolRecord::Regex { .. } => SymbolKind::Regex,
        }
    }

    /// The raw quoted literal when this symbol is a terminal.
    pub fn as_terminal(&self) -> Option<StringLiteralSyntax<'a>> {
        match self.record {
            SymbolRecord::Terminal { span } => {
                Some(StringLiteralSyntax::new(self.doc, span.clone()))
            }
            SymbolRecord::NonTerminal(_) | SymbolRecord::Regex { .. } => None,
        }
    }

    /// The non-terminal view when this symbol is a non-terminal reference.
    pub fn as_non_terminal(&self) -> Option<NonTerminalSyntax<'a>> {
        match self.record {
            SymbolRecord::NonTerminal(record) => Some(NonTerminalSyntax::new(self.doc, record)),
            SymbolRecord::Terminal { .. } | SymbolRecord::Regex { .. } => None,
        }
    }

    /// The regular-expression form when this symbol is `re("pattern")`.
    pub fn as_regex(&self) -> Option<RegexSyntax<'a>> {
        match self.record {
            SymbolRecord::Regex { span, pattern } => Some(RegexSyntax {
                doc: self.doc,
                span: span.clone(),
                pattern: pattern.clone(),
            }),
            SymbolRecord::Terminal { .. } | SymbolRecord::NonTerminal(_) => None,
        }
    }
}

/// A raw quoted string token used as a terminal, type, or regex pattern.
#[derive(Debug, Clone)]
pub struct StringLiteralSyntax<'a> {
    doc: &'a ParsedDocument,
    span: Range<usize>,
}

impl<'a> StringLiteralSyntax<'a> {
    fn new(doc: &'a ParsedDocument, span: Range<usize>) -> Self {
        Self { doc, span }
    }

    pub fn range(&self) -> Range<usize> {
        self.span.clone()
    }

    /// Exact source spelling, including quotes and undecoded escapes.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.span.clone())
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

    pub fn lower_bound(&self) -> IntegerSyntax<'a> {
        IntegerSyntax::new(self.doc, self.record.lower.clone())
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
}

/// Raw syntax for a regular-expression symbol, `re("pattern")`.
#[derive(Debug, Clone)]
pub struct RegexSyntax<'a> {
    doc: &'a ParsedDocument,
    span: Range<usize>,
    pattern: Range<usize>,
}

impl<'a> RegexSyntax<'a> {
    pub fn range(&self) -> Range<usize> {
        self.span.clone()
    }

    pub fn text(&self) -> &'a str {
        self.doc.slice(self.span.clone())
    }

    /// The quoted, uncompiled regex pattern.
    pub fn pattern(&self) -> StringLiteralSyntax<'a> {
        StringLiteralSyntax::new(self.doc, self.pattern.clone())
    }
}
