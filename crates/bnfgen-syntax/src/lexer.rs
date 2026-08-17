//! Total lexing into the document's token buffer.
//!
//! Lexing can never fail the document: every character position either
//! produces a token (significant or trivia) or a retained
//! [`UnrecognizedInput`](crate::SyntaxErrorKind::UnrecognizedInput) error,
//! and iteration always continues past the problem. This is the lexical half
//! of "parsing is total"; the grammar-recognition half lives in
//! `parser.lalrpop` and `document.rs`.

use logos::Logos;

use crate::error::{SyntaxError, SyntaxErrorKind};
use crate::token::{SyntaxToken, TokenKind};

/// Lex `source` into a source-ordered token buffer plus retained lexical
/// errors.
///
/// Logos reports unmatched input as `Err(())` with the span it consumed —
/// at least one byte, so the iterator is guaranteed to make progress and
/// this loop terminates for every input. Invalid text is recorded and
/// skipped here; ticket 02 replaces the skip with retained token
/// representation so that even lexical failures stay observable in the
/// buffer.
pub(super) fn lex(source: &str) -> (Vec<SyntaxToken>, Vec<SyntaxError>) {
    let mut tokens = Vec::new();
    let mut errors = Vec::new();

    for (result, span) in TokenKind::lexer(source).spanned() {
        match result {
            Ok(kind) => tokens.push(SyntaxToken::new(kind, span.start..span.end)),
            Err(()) => errors.push(SyntaxError::new(
                SyntaxErrorKind::UnrecognizedInput,
                span.start..span.end,
            )),
        }
    }

    (tokens, errors)
}
