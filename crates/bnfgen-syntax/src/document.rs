//! The parsed document: the single source of truth a caller holds.
//!
//! A `ParsedDocument` owns the original source, the complete token buffer,
//! retained syntax errors, and private records for whatever the grammar
//! recognized. Everything else in this crate is a borrowing view over it.
//! Because the document owns plain data, it is `Send + Sync` and can be
//! retained across worker threads (asserted through the public crate).

use std::fmt;
use std::ops::Range;

use crate::error::{SyntaxError, SyntaxErrorKind};
use crate::lexer;
use crate::records::RuleRecord;
use crate::token::SyntaxToken;
use crate::views::RuleSyntax;

/// The result of parsing: source, tokens, errors, and recognized structure
/// for any input — valid, empty, or malformed.
///
/// Construct through [`parse`](crate::parse).
pub struct ParsedDocument {
    source: Box<str>,
    tokens: Vec<SyntaxToken>,
    errors: Vec<SyntaxError>,
    rules: Vec<RuleRecord>,
}

/// Parse `source` into a document. See [`crate::parse`].
pub(super) fn parse(source: &str) -> ParsedDocument {
    let (tokens, mut errors) = lexer::lex(source);

    // The significant-token adapter: LALRPOP consumes only grammar-relevant
    // tokens while trivia stays in the complete buffer (ADR 0003). Lexical
    // failures were already diverted into `errors`, so this stream is
    // infallible.
    let significant = tokens
        .iter()
        .filter(|token| !token.kind().is_trivia())
        .map(|token| Ok((token.range().start, token.kind(), token.range().end)));

    let rules = match crate::parser::DocumentParser::new().parse(significant) {
        Ok(rules) => rules,
        Err(error) => {
            errors.push(to_syntax_error(error));
            // Ticket 04 replaces this whole-document drop with recovery
            // that resynchronizes to later rules. Until then, a document
            // with a grammar error recognizes no rules — but its source and
            // complete token buffer remain queryable.
            Vec::new()
        }
    };

    ParsedDocument {
        source: source.to_owned().into_boxed_str(),
        tokens,
        errors,
        rules,
    }
}

/// Translate a LALRPOP failure into retained, structured data. The
/// generated parser's `expected` terminal list is LALRPOP display text and
/// is deliberately dropped here.
///
/// TODO(ticket 04): surface structured expected-terminal information once
/// recovery needs it, instead of resurrecting message strings.
fn to_syntax_error(
    error: lalrpop_util::ParseError<usize, crate::token::TokenKind, &'static str>,
) -> SyntaxError {
    use lalrpop_util::ParseError;

    match error {
        ParseError::InvalidToken { location } => {
            SyntaxError::new(SyntaxErrorKind::UnrecognizedInput, location..location)
        }
        ParseError::UnrecognizedEof { location, .. } => {
            SyntaxError::new(SyntaxErrorKind::UnexpectedEof, location..location)
        }
        ParseError::UnrecognizedToken {
            token: (start, _, end),
            ..
        } => SyntaxError::new(SyntaxErrorKind::UnexpectedToken, start..end),
        ParseError::ExtraToken {
            token: (start, _, end),
        } => SyntaxError::new(SyntaxErrorKind::UnexpectedToken, start..end),
        ParseError::User { .. } => {
            unreachable!("syntax actions are infallible; user errors cannot occur")
        }
    }
}

impl fmt::Debug for ParsedDocument {
    /// Deliberately summary-only: `Debug` on public types is not a snapshot
    /// contract (the spec requires normalized representations instead), so
    /// this reports shape, not contents.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ParsedDocument")
            .field("source_bytes", &self.source.len())
            .field("tokens", &self.tokens.len())
            .field("rules", &self.rules.len())
            .field("errors", &self.errors.len())
            .finish()
    }
}

impl ParsedDocument {
    /// The original source, retained verbatim.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// All tokens in source order — significant tokens and trivia alike.
    /// A token's text is not stored on it; slice `source()` with the
    /// token's range, or use [`slice`](Self::slice).
    pub fn tokens(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        self.tokens.iter().cloned()
    }

    /// The recognized rules, in source order.
    pub fn rules(&self) -> impl Iterator<Item = RuleSyntax<'_>> + '_ {
        self.rules
            .iter()
            .map(|record| RuleSyntax::new(self, record))
    }

    /// The retained syntax errors, in the order they were discovered.
    pub fn errors(&self) -> &[SyntaxError] {
        &self.errors
    }

    /// Source text for a UTF-8 byte range previously obtained from this
    /// document.
    ///
    /// # Panics
    ///
    /// Panics if the range is empty-reversed or extends past the source.
    /// Ranges handed out by this crate are always in bounds; this exists to
    /// fail fast on caller arithmetic mistakes.
    pub fn slice(&self, range: Range<usize>) -> &str {
        assert!(
            range.start <= range.end && range.end <= self.source.len(),
            "range {range:?} is not within the {}-byte source",
            self.source.len(),
        );
        &self.source[range]
    }
}
