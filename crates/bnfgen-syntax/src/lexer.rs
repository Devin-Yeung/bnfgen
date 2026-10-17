//! Total lexing into the document's token buffer.
//!
//! Lexing can never fail the document: every byte of the source lands in
//! exactly one token. Recognized input produces a significant or trivia
//! kind; unmatched input produces a retained recovery kind
//! ([`Invalid`](crate::TokenKind::Invalid) or
//! [`UnterminatedStr`](crate::TokenKind::UnterminatedStr)) plus a matching
//! [`SyntaxDiagnostic`](crate::SyntaxDiagnostic) recording the failure, and iteration
//! always continues past the problem. This is the lexical half of "parsing
//! is total"; the grammar-recognition half lives in the private parser
//! module and `document.rs`.

use logos::Logos;

use crate::error::{SyntaxDiagnostic, SyntaxDiagnosticKind};
use crate::token::{SyntaxToken, TokenKind};

/// Lex `source` into a source-ordered token buffer plus retained lexical
/// diagnostics, in the same source order as the tokens.
///
/// Logos reports unmatched input as `Err(())` with the span it consumed —
/// at least one byte, so the iterator is guaranteed to make progress and
/// this loop terminates for every input. Rather than dropping unmatched
/// bytes, each run is retained as a recovery token so the buffer covers
/// the source completely, with a matching error recording the failure.
pub(super) fn lex(source: &str) -> (Vec<SyntaxToken>, Vec<SyntaxDiagnostic>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    for (result, span) in TokenKind::lexer(source).spanned() {
        match result {
            Ok(kind) => tokens.push(SyntaxToken::new(kind, span.start..span.end)),
            Err(()) => {
                // An unmatched run begins either with `"` — making it the
                // residue of a string literal the lexer could not terminate,
                // since `Str` is the only rule that can start a match there
                // and the match died at end of file or on an escape `Str`
                // rejects — or with a byte no rule accepts at all. The
                // span is never empty, so indexing its first byte is safe.
                let range = span.start..span.end;
                let (kind, error_kind) = if source.as_bytes()[range.start] == b'"' {
                    (
                        TokenKind::UnterminatedStr,
                        SyntaxDiagnosticKind::UnterminatedString,
                    )
                } else {
                    (TokenKind::Invalid, SyntaxDiagnosticKind::UnrecognizedInput)
                };
                tokens.push(SyntaxToken::new(kind, range.clone()));
                errors.push(SyntaxDiagnostic::new(error_kind, range));
            }
        }
    }

    (tokens, errors)
}
