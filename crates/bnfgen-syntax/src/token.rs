//! The lexical layer of the Bnfgen source language.
//!
//! Two invariants govern this module:
//!
//! 1. **Lexemes stay raw.** Integer, string, identifier, and type tokens
//!    record only a kind and a byte range. Decoding escapes, parsing
//!    integers, and validating spellings are semantic concerns that belong
//!    downstream; the syntax crate never destroys the original spelling.
//!    This is why none of the variants below carry data — contrast with the
//!    legacy lexer in `bnfgen-core`, which decoded strings and integers at
//!    lex time.
//! 2. **The token buffer is the source-preserving representation.**
//!    Whitespace and comments are real token kinds, not `skip` patterns, and
//!    lexical failures are retained as recovery kinds (`Invalid`,
//!    `UnterminatedStr`), so the buffer tiles the source completely: every
//!    byte belongs to exactly one token, and token iteration can reconstruct
//!    what the user sees — including the broken parts.

use std::ops::Range;

use logos::Logos;

/// The kind of a lexed token.
///
/// The significant kinds mirror the legacy `bnfgen-core` lexer one-to-one so
/// that valid grammars keep lexing identically during migration. The trivia
/// kinds (`Whitespace`, `Comment`) are new: the legacy lexer skipped them.
#[derive(Logos, Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("|")]
    Or,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::=")]
    Def,
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,
    #[token(";")]
    Semi,
    #[token("re")]
    Re,
    /// An integer literal, kept raw. Priority 2 so that all-digit input
    /// lexes as `Int` rather than falling to the `Id` regex.
    #[regex(r"[0-9]+", priority = 2)]
    Int,
    /// An identifier. The regex is copied verbatim from the legacy lexer,
    /// quirks included (`-_` is parsed by the regex engine as the range
    /// `A-_`, not as two literals), so identifier acceptance cannot drift
    /// during the migration.
    #[regex(r"[a-zA-Z-_0-9]+", priority = 1)]
    Id,
    /// A string literal including its quotes, kept raw: escapes are not
    /// decoded here. An unterminated string does not match this pattern;
    /// the unmatched run is retained as [`TokenKind::UnterminatedStr`].
    #[regex(r#""(\\["nrt\\]|[^"\\])*""#)]
    Str,
    /// Horizontal and vertical spacing. The character set matches the
    /// legacy skip pattern exactly: `\r` is deliberately absent, so a bare
    /// `\r` is drift-free invalid input — the legacy lexer rejected CRLF
    /// files, and this crate keeps that behavior, now observable as an
    /// [`Invalid`](TokenKind::Invalid) token rather than a dropped byte.
    /// (`\r` inside a string literal is legal; `Str`'s `[^"\\]` class
    /// covers it.)
    #[regex(r"[ \t\n\f]+")]
    Whitespace,
    /// A line comment. It extends to (not including) the newline, so a
    /// comment at end of file still lexes — the newline belongs to
    /// `Whitespace` when present. Unlike the legacy skip pattern
    /// `//[^\n]*?\n`, which required a trailing newline and thus could not
    /// consume a comment at end of file, this pattern is EOF-safe.
    /// `allow_greedy` is safe here: the class cannot cross the `\n` that
    /// bounds it.
    #[regex(r"//[^\n]*", allow_greedy = true)]
    Comment,
    /// Input that does not match any token rule, retained verbatim in the
    /// token buffer in source order. The legacy lexer reported this class
    /// of input as a lexical error and lost the bytes; here they stay
    /// observable, with a matching
    /// [`UnrecognizedInput`](crate::SyntaxDiagnosticKind::UnrecognizedInput)
    /// error recording the failure.
    ///
    /// Deliberately carries no logos pattern: a catch-all regex would tie
    /// with single-byte literals under logos' length-times-two literal
    /// priority and make lexing declaration-order-dependent. The lexer
    /// constructs this kind from unmatched spans instead.
    Invalid,
    /// The residue of a string literal the lexer could not terminate: a
    /// run beginning with the opening quote that never found its closing
    /// quote before the match died — at end of file, or on an escape
    /// `Str` does not accept. Retained raw with a matching
    /// [`UnterminatedString`](crate::SyntaxDiagnosticKind::UnterminatedString)
    /// error. Like [`Invalid`](TokenKind::Invalid), constructed by
    /// `crate::lexer`, never by logos.
    UnterminatedStr,
}

impl TokenKind {
    /// Whether this kind is trivia: present in the token buffer for source
    /// fidelity, but never significant to grammar recognition.
    pub fn is_trivia(&self) -> bool {
        matches!(self, TokenKind::Whitespace | TokenKind::Comment)
    }

    /// Whether this kind is a retained lexical failure: bytes kept in the
    /// buffer as recoverable input that the grammar never consumes. Every
    /// such token is accompanied by a [`SyntaxDiagnostic`](crate::SyntaxDiagnostic)
    /// recording the failure.
    pub fn is_lexical_recovery(&self) -> bool {
        matches!(self, TokenKind::Invalid | TokenKind::UnterminatedStr)
    }

    /// Whether this kind participates in grammar recognition — the
    /// significant-token stream the tolerant parser consumes.
    ///
    /// An unterminated string participates because its opening quote is a
    /// reliable typed fact for a terminal, type, or regex pattern. Arbitrary
    /// invalid input remains outside structural recognition; both kinds stay
    /// in the complete token buffer and retain their lexical errors.
    pub fn is_significant(&self) -> bool {
        !self.is_trivia() && !matches!(self, TokenKind::Invalid)
    }
}

/// A token in a parsed document: a kind plus the UTF-8 byte range it
/// occupies in the retained source.
///
/// The token carries no text. Its raw lexeme is obtained by slicing the
/// document source with [`range`](Self::range) — one canonical copy of the
/// text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxToken {
    kind: TokenKind,
    range: Range<usize>,
}

impl SyntaxToken {
    pub(super) fn new(kind: TokenKind, range: Range<usize>) -> Self {
        Self { kind, range }
    }

    pub fn kind(&self) -> TokenKind {
        self.kind
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}
