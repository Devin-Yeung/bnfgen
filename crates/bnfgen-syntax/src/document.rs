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
use crate::token::{SyntaxToken, TokenKind};
use crate::views::RuleSyntax;

/// A private result from one grammar-level rule attempt.
///
/// LALRPOP owns the decision to synchronize and the tokens it drops. The
/// document converts that event into stable public errors and ranges without
/// exposing LALRPOP's recovery type to callers.
pub(crate) enum ParseEvent {
    Rule(RuleRecord),
    Recovery {
        error: SyntaxError,
        range: Range<usize>,
    },
}

impl ParseEvent {
    /// Package LALRPOP's private recovery data at the grammar boundary.
    pub(crate) fn recovered(
        recovery: lalrpop_util::ErrorRecovery<usize, TokenKind, &'static str>,
        range: Range<usize>,
    ) -> Self {
        Self::Recovery {
            error: to_syntax_error(recovery.error),
            range,
        }
    }
}

/// The result of parsing: source, tokens, errors, and recognized structure
/// for any input — valid, empty, or malformed.
///
/// Construct through [`parse`](crate::parse).
pub struct ParsedDocument {
    source: Box<str>,
    tokens: Vec<SyntaxToken>,
    errors: Vec<SyntaxError>,
    recovery_ranges: Vec<Range<usize>>,
    rules: Vec<RuleRecord>,
}

/// Parse `source` into a document. See [`crate::parse`].
pub(super) fn parse(source: &str) -> ParsedDocument {
    let (tokens, mut errors) = lexer::lex(source);
    let mut recovery_ranges = tokens
        .iter()
        .filter(|token| token.kind().is_lexical_recovery())
        .map(SyntaxToken::range)
        .collect::<Vec<_>>();

    // Whole-document parse over significant tokens. The grammar's `!`
    // production synchronizes after a malformed rule.
    let significant = tokens
        .iter()
        .filter(|token| token.kind().is_significant())
        .map(|token| Ok((token.range().start, token.kind(), token.range().end)));

    let mut rules = Vec::new();
    match crate::parser::DocumentParser::new().parse(significant) {
        Ok(events) => {
            for event in events {
                match event {
                    ParseEvent::Rule(rule) if rule_has_lexical_failure(&tokens, &rule) => {
                        recovery_ranges.push(rule.span);
                    }
                    ParseEvent::Rule(rule) => rules.push(rule),
                    ParseEvent::Recovery { error, range } => {
                        errors.push(error);
                        recovery_ranges.push(range);
                    }
                }
            }
        }
        // EOF can still be unrecoverable when no synchronization token is
        // available. The public operation remains total; its precise error
        // range is also the only truthful recovery region in that case.
        Err(error) => {
            let error = to_syntax_error(error);
            recovery_ranges.push(error.range());
            errors.push(error);
        }
    }
    let recovery_ranges = normalize_recovery_ranges(recovery_ranges);

    ParsedDocument {
        source: source.to_owned().into_boxed_str(),
        tokens,
        errors,
        recovery_ranges,
        rules,
    }
}

/// Whether a grammar-recognized rule overlaps retained lexical failure.
///
/// The grammar does not consume lexical recovery tokens. Publishing a rule
/// across one would therefore pretend that deleting the invalid text was a
/// valid edit, which is not a truthful syntax fact.
fn rule_has_lexical_failure(tokens: &[SyntaxToken], rule: &RuleRecord) -> bool {
    tokens.iter().any(|token| {
        token.kind().is_lexical_recovery()
            && token.range().start < rule.span.end
            && rule.span.start < token.range().end
    })
}

/// Normalize ranges collected independently by the lexer and parser.
fn normalize_recovery_ranges(mut ranges: Vec<Range<usize>>) -> Vec<Range<usize>> {
    ranges.retain(|range| range.start < range.end);
    ranges.sort_unstable_by_key(|range| range.start);

    let mut normalized: Vec<Range<usize>> = Vec::new();
    for range in ranges {
        if let Some(previous) = normalized.last_mut() {
            if range.start <= previous.end {
                previous.end = previous.end.max(range.end);
                continue;
            }
        }
        normalized.push(range);
    }
    normalized
}

/// Translate a LALRPOP failure into retained, structured data. The
/// generated parser's `expected` terminal list is LALRPOP display text and
/// is deliberately dropped here.
///
/// TODO: surface structured expected-terminal information once recovery needs
/// it, instead of resurrecting message strings.
fn to_syntax_error(
    error: lalrpop_util::ParseError<usize, crate::token::TokenKind, &'static str>,
) -> SyntaxError {
    use lalrpop_util::ParseError;

    match error {
        ParseError::InvalidToken { location } => {
            // A significant kind the grammar does not map (an `Int`, say):
            // the input *was* recognized as a
            // token, so this is a grammar rejection, not unrecognized
            // input — `UnrecognizedInput` belongs to `Invalid` tokens
            // alone. Once every significant kind has a production, this arm
            // becomes unreachable.
            SyntaxError::new(SyntaxErrorKind::UnexpectedToken, location..location)
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
            .field("recovery_ranges", &self.recovery_ranges.len())
            .finish()
    }
}

impl ParsedDocument {
    /// The original source, retained verbatim.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// All tokens in source order — significant tokens, trivia, and
    /// retained lexical failures alike; together they cover every byte of
    /// the source. A token's text is not stored on it; slice `source()`
    /// with the token's range, or use [`slice`](Self::slice).
    pub fn tokens(&self) -> impl Iterator<Item = SyntaxToken> + '_ {
        self.tokens.iter().cloned()
    }

    /// The token at `offset`, searched over the complete buffer.
    ///
    /// Total over `0..=source.len()` and never panics. The token whose
    /// `start <= offset < end` contains `offset`; an offset where two
    /// tokens meet resolves to the token that starts there;
    /// `offset == source.len()` resolves to the final token. Returns
    /// `None` past the end of the source and for an empty document.
    ///
    /// These boundary rules are the ones cursor-context classification builds
    /// on; keep them this simple.
    pub fn token_at(&self, offset: usize) -> Option<SyntaxToken> {
        if offset > self.source.len() {
            return None;
        }
        // Token starts are non-decreasing (the buffer tiles the source in
        // order), so `partition_point` finds the first token starting
        // after `offset`; the token before it is the containment
        // candidate.
        let next = self
            .tokens
            .partition_point(|token| token.range().start <= offset);
        if next > 0 && offset < self.tokens[next - 1].range().end {
            return Some(self.tokens[next - 1].clone());
        }
        if next < self.tokens.len() && self.tokens[next].range().start == offset {
            return Some(self.tokens[next].clone());
        }
        // Not inside any token: reachable only at end of source (the
        // buffer tiles the source, so genuine gaps do not exist), where
        // the final token is the answer.
        self.tokens.last().cloned()
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

    /// Source ranges skipped while recovering from malformed input.
    ///
    /// Ranges are source ordered, non-overlapping, and safe to pass to
    /// [`slice`](Self::slice). A lexical failure within an otherwise complete
    /// rule recovers that whole rule, so callers never mistake it for a
    /// complete generation-ready fact.
    pub fn recovery_ranges(&self) -> impl Iterator<Item = Range<usize>> + '_ {
        self.recovery_ranges.iter().cloned()
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
