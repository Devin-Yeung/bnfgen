//! The lossless syntax snapshot produced for one source string.

use std::fmt;
use std::ops::Range;

use crate::error::SyntaxDiagnostic;
use crate::lexer;
use crate::parser;
use crate::records::RuleRecord;
use crate::recovery::RecoveryObservation;
use crate::token::SyntaxToken;
use crate::views::RuleSyntax;

/// The total result of syntax parsing.
///
/// A document is not a semantic grammar: it retains source, source tokens,
/// source-established syntax facts, diagnostics, and recovery observations.
/// Analysis derives names, graph edges, and generation eligibility later.
pub struct ParsedDocument {
    source: Box<str>,
    tokens: Vec<SyntaxToken>,
    diagnostics: Vec<SyntaxDiagnostic>,
    recovery: Vec<RecoveryObservation>,
    rules: Vec<RuleRecord>,
}

pub(super) fn parse(source: &str) -> ParsedDocument {
    let (tokens, lexical_diagnostics) = lexer::lex(source);
    let output = parser::parse(&tokens, lexical_diagnostics);
    ParsedDocument {
        source: source.to_owned().into_boxed_str(),
        tokens,
        diagnostics: output.diagnostics,
        recovery: output.recovery,
        rules: output.rules,
    }
}

impl fmt::Debug for ParsedDocument {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedDocument")
            .field("source_bytes", &self.source.len())
            .field("tokens", &self.tokens.len())
            .field("rules", &self.rules.len())
            .field("diagnostics", &self.diagnostics.len())
            .field("recovery", &self.recovery.len())
            .finish()
    }
}

impl ParsedDocument {
    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn tokens(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        self.tokens.iter().cloned()
    }

    pub fn token_at(&self, offset: usize) -> Option<SyntaxToken> {
        if offset > self.source.len() {
            return None;
        }
        let next = self
            .tokens
            .partition_point(|token| token.range().start <= offset);
        if next > 0 && offset < self.tokens[next - 1].range().end {
            return Some(self.tokens[next - 1].clone());
        }
        if next < self.tokens.len() && self.tokens[next].range().start == offset {
            return Some(self.tokens[next].clone());
        }
        self.tokens.last().cloned()
    }

    pub fn rules(&self) -> impl Iterator<Item = RuleSyntax<'_>> + '_ {
        self.rules
            .iter()
            .map(|record| RuleSyntax::new(self, record))
    }

    pub fn diagnostics(&self) -> &[SyntaxDiagnostic] {
        &self.diagnostics
    }

    pub fn recovery(&self) -> impl Iterator<Item = &RecoveryObservation> {
        self.recovery.iter()
    }

    pub fn slice(&self, range: Range<usize>) -> &str {
        assert!(
            range.start <= range.end && range.end <= self.source.len(),
            "range {range:?} is not within the {}-byte source",
            self.source.len(),
        );
        &self.source[range]
    }
}
