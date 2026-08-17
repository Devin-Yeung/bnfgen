//! Language-specific immutable views over a parsed document.
//!
//! Views borrow the document and wrap its private records, so callers can
//! never retain syntax independently of the source it came from (ADR 0003).
//! Every accessor returns either raw text sliced from the document or a
//! UTF-8 byte range — never a decoded or validated value. Accessors for
//! required children become `Option` returning once recovery can produce
//! partially recognized rules (tickets 04/05); today a view exists only for
//! fully recognized syntax.

use std::ops::Range;

use crate::document::ParsedDocument;
use crate::records::{AlternativeRecord, RuleRecord, SymbolRecord};

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
        NonTerminalSyntax::new(self.doc, self.record.lhs.clone(), self.record.name.clone())
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
    /// The bracketed form, `<name>` — later also `<name: "type">`.
    span: Range<usize>,
    /// The identifier inside the brackets.
    name: Range<usize>,
}

impl<'a> NonTerminalSyntax<'a> {
    pub(super) fn new(doc: &'a ParsedDocument, span: Range<usize>, name: Range<usize>) -> Self {
        Self { doc, span, name }
    }

    /// The full bracketed form's source range.
    pub fn range(&self) -> Range<usize> {
        self.span.clone()
    }

    /// The full bracketed form's raw text, delimiters included.
    pub fn text(&self) -> &'a str {
        self.doc.slice(self.span.clone())
    }

    /// The bare identifier's source range.
    pub fn name_range(&self) -> Range<usize> {
        self.name.clone()
    }

    /// The identifier's raw spelling.
    pub fn name(&self) -> &'a str {
        self.doc.slice(self.name.clone())
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

    /// The alternative's symbols, in source order.
    pub fn symbols(&self) -> impl Iterator<Item = SymbolSyntax<'a>> + 'a {
        let doc = self.doc;
        self.record
            .symbols
            .iter()
            .map(move |record| SymbolSyntax::new(doc, record))
    }
}

/// The form a symbol takes. Payloads are reached through the typed view
/// methods on [`SymbolSyntax`] rather than carried here, keeping the kind
/// cheap to match on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SymbolKind {
    Terminal,
    NonTerminal,
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
            SymbolRecord::NonTerminal { span, .. } => span.clone(),
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
            SymbolRecord::NonTerminal { .. } => SymbolKind::NonTerminal,
        }
    }

    /// The non-terminal view when this symbol is a non-terminal reference.
    pub fn as_non_terminal(&self) -> Option<NonTerminalSyntax<'a>> {
        match self.record {
            SymbolRecord::Terminal { .. } => None,
            SymbolRecord::NonTerminal { span, name } => {
                Some(NonTerminalSyntax::new(self.doc, span.clone(), name.clone()))
            }
        }
    }
}
