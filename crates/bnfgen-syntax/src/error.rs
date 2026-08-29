//! Syntax diagnostics retained by a parsed document.
//!
//! A diagnostic explains an observed syntax problem. It is distinct from a
//! recovery observation: a missing delimiter has a diagnostic but no source
//! range to recover, while unexpected source can have both a diagnostic and
//! a recovery observation.

use std::ops::Range;

/// A source-ranged problem observed while lexing or parsing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxDiagnostic {
    kind: SyntaxDiagnosticKind,
    range: Range<usize>,
}

impl SyntaxDiagnostic {
    pub(crate) fn new(kind: SyntaxDiagnosticKind, range: Range<usize>) -> Self {
        Self { kind, range }
    }

    pub fn kind(&self) -> SyntaxDiagnosticKind {
        self.kind
    }

    /// The UTF-8 byte range the diagnostic refers to.
    ///
    /// A missing construct has an empty range at its insertion point. An
    /// unexpected source construct has the nonempty range it occupies.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}

/// The source-language category of a syntax diagnostic.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxDiagnosticKind {
    /// Input did not match any lexical token rule.
    UnrecognizedInput,
    /// A string literal has no closing quote.
    UnterminatedString,
    /// A token was present but cannot occur at this syntactic position.
    UnexpectedToken,
    /// A syntactic element was absent at the diagnostic's insertion point.
    Missing(ExpectedSyntax),
}

/// A language-specific syntactic element that a parser expected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExpectedSyntax {
    RuleName,
    NonTerminalName,
    NonTerminalClosingDelimiter,
    Definition,
    Alternative,
    Symbol,
    RepeatLowerBound,
    RepeatClosingDelimiter,
    RegexOpeningParenthesis,
    RegexPattern,
    RegexClosingParenthesis,
    RuleTerminator,
}
