//! Tolerant recursive-descent parser over the significant token stream.
//!
//! The shape follows Ruff's parser:
//!
//! - productions always return their records; missing required syntax is an
//!   absent child plus a diagnostic at the insertion point;
//! - unexpected source is consumed into the nearest owning record's `skipped`
//!   list, which is the only place recovery is recorded;
//! - parse errors go to a side list that is merged with lexical errors by
//!   source position;
//! - every loop asserts that it consumes input, and a loop that stops
//!   consuming panics instead of spinning.
//!
//! The grammar lives in `parser::grammar`; this module holds the cursor, the
//! recovery helpers, and the merge of diagnostics.

mod grammar;

use std::ops::Range;

use crate::error::{ExpectedSyntax, SyntaxDiagnostic, SyntaxDiagnosticKind};
use crate::records::{Skipped, SourceFile};
use crate::recovery::RecoveryKind;
use crate::token::{SyntaxToken, TokenKind};

pub(crate) struct ParseOutput {
    pub(crate) source_file: SourceFile,
    pub(crate) diagnostics: Vec<SyntaxDiagnostic>,
}

pub(crate) fn parse(
    tokens: &[SyntaxToken],
    lexical_diagnostics: Vec<SyntaxDiagnostic>,
) -> ParseOutput {
    let significant: Vec<&SyntaxToken> = tokens
        .iter()
        .filter(|token| token.kind().is_significant())
        .collect();
    let mut parser = Parser {
        tokens: &significant,
        cursor: 0,
        errors: Vec::new(),
    };
    let source_file = parser.source_file();
    ParseOutput {
        source_file,
        diagnostics: merge_diagnostics(lexical_diagnostics, parser.errors),
    }
}

/// Merges lexical and parse errors into one list in source order.
///
/// On equal start positions a lexical error comes first. An unexpected-token
/// error on the same token as a lexical error repeats the lexical error, so it
/// is dropped.
fn merge_diagnostics(
    lexical: Vec<SyntaxDiagnostic>,
    parser: Vec<SyntaxDiagnostic>,
) -> Vec<SyntaxDiagnostic> {
    let lexical_starts: Vec<usize> = lexical
        .iter()
        .map(|diagnostic| diagnostic.range().start)
        .collect();
    let mut merged = lexical;
    for diagnostic in parser {
        let start = diagnostic.range().start;
        let repeats_lexical = diagnostic.kind() == SyntaxDiagnosticKind::UnexpectedToken
            && lexical_starts.contains(&start);
        if !repeats_lexical {
            merged.push(diagnostic);
        }
    }
    merged.sort_by_key(|diagnostic| diagnostic.range().start);
    merged
}

/// A set of token kinds, stored as a bit mask over the kind discriminants.
#[derive(Debug, Clone, Copy)]
struct TokenSet(u32);

impl TokenSet {
    const fn new(kinds: &[TokenKind]) -> Self {
        let mut bits = 0;
        let mut index = 0;
        while index < kinds.len() {
            bits |= 1 << kinds[index] as u32;
            index += 1;
        }
        Self(bits)
    }

    fn contains(self, kind: TokenKind) -> bool {
        self.0 & (1 << kind as u32) != 0
    }
}

/// Detects a loop that stopped consuming input.
#[derive(Debug, Default)]
struct Progress(Option<usize>);

impl Progress {
    /// Asserts that the cursor moved since the previous call.
    ///
    /// # Panics
    ///
    /// Panics if the parser is still at the token it was at last time.
    fn assert_progressing(&mut self, parser: &Parser<'_>) {
        assert_ne!(
            self.0,
            Some(parser.cursor),
            "parser is not progressing at token {}",
            parser.cursor
        );
        self.0 = Some(parser.cursor);
    }
}

struct Parser<'a> {
    tokens: &'a [&'a SyntaxToken],
    cursor: usize,
    errors: Vec<SyntaxDiagnostic>,
}

impl Parser<'_> {
    // ----- cursor --------------------------------------------------------

    fn current_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.cursor).map(|token| token.kind())
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current_kind() == Some(kind)
    }

    fn at_any(&self, set: TokenSet) -> bool {
        self.current_kind().is_some_and(|kind| set.contains(kind))
    }

    fn at_eof(&self) -> bool {
        self.cursor == self.tokens.len()
    }

    fn at_string(&self) -> bool {
        self.at(TokenKind::Str) || self.at(TokenKind::UnterminatedStr)
    }

    fn at_symbol_start(&self) -> bool {
        self.at_string() || self.at(TokenKind::LAngle) || self.at(TokenKind::Re)
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    fn eat_range(&mut self, kind: TokenKind) -> Option<Range<usize>> {
        self.at(kind).then(|| self.bump())
    }

    /// Consumes the current token and returns its range.
    fn bump(&mut self) -> Range<usize> {
        let token = self
            .tokens
            .get(self.cursor)
            .expect("parser bumps only at a token");
        self.cursor += 1;
        token.range()
    }

    /// The start of the current token, or the insertion point at end of file.
    fn current_start(&self) -> usize {
        match self.tokens.get(self.cursor) {
            Some(token) => token.range().start,
            None => self.insertion_point(),
        }
    }

    /// The end of the last consumed token: where a missing token would go.
    fn insertion_point(&self) -> usize {
        self.cursor
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .map_or(0, |token| token.range().end)
    }

    /// The end of the last consumed token, but never before `start`. An empty
    /// production therefore gets an empty range at its start.
    fn previous_end(&self, start: usize) -> usize {
        self.insertion_point().max(start)
    }

    /// Whether the cursor is at a `<` that begins a complete rule header:
    /// `<`, identifier, optional `:` and a terminated string, `>`, `::=`.
    ///
    /// A bare `<` with an identifier does not qualify, so an incomplete
    /// reference stays with its enclosing rule.
    fn at_rule_start(&self) -> bool {
        let kind_at = |index: usize| self.tokens.get(index).map(|token| token.kind());
        let mut index = self.cursor;

        if kind_at(index) != Some(TokenKind::LAngle) || kind_at(index + 1) != Some(TokenKind::Id) {
            return false;
        }
        index += 2;

        if kind_at(index) == Some(TokenKind::Colon) {
            if kind_at(index + 1) != Some(TokenKind::Str) {
                return false;
            }
            index += 2;
        }

        kind_at(index) == Some(TokenKind::RAngle) && kind_at(index + 1) == Some(TokenKind::Def)
    }

    // ----- diagnostics and recovery -------------------------------------

    /// Reports missing syntax at the insertion point without inventing tokens.
    fn missing(&mut self, expected: ExpectedSyntax) {
        let offset = self.insertion_point();
        self.errors.push(SyntaxDiagnostic::new(
            SyntaxDiagnosticKind::Missing(expected),
            offset..offset,
        ));
    }

    /// Consumes the current token into `skipped` and reports it.
    fn skip_one(&mut self, skipped: &mut Vec<Skipped>) {
        let kind = self.current_kind().expect("skipping only at a token");
        let range = self.bump();
        self.errors.push(SyntaxDiagnostic::new(
            SyntaxDiagnosticKind::UnexpectedToken,
            range.clone(),
        ));
        skipped.push(Skipped {
            range,
            kind: recovery_kind(kind),
        });
    }

    /// Skips tokens until the owner's `stop` set, a rule header, or the end of
    /// the file. Each skipped token is recorded in `skipped`.
    fn skip_until(&mut self, stop: TokenSet, skipped: &mut Vec<Skipped>) {
        let mut progress = Progress::default();
        while !self.at_eof() && !self.at_rule_start() && !self.at_any(stop) {
            progress.assert_progressing(self);
            self.skip_one(skipped);
        }
    }
}

/// How a token the parser could not attach is classified for recovery.
fn recovery_kind(kind: TokenKind) -> RecoveryKind {
    match kind {
        TokenKind::Invalid => RecoveryKind::InvalidLexeme,
        TokenKind::UnterminatedStr => RecoveryKind::UnterminatedString,
        _ => RecoveryKind::UnexpectedToken,
    }
}
