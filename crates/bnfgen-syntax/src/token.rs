//! The lexical layer of the Bnfgen source language.
//!
//! Two invariants govern this module, both taken from
//! `docs/adr/0003-use-token-backed-tolerant-syntax-documents.md`:
//!
//! 1. **Lexemes stay raw.** Integer, string, identifier, and type tokens
//!    record only a kind and a byte range. Decoding escapes, parsing
//!    integers, and validating spellings are semantic concerns that belong
//!    downstream; the syntax crate never destroys the original spelling.
//!    This is why none of the variants below carry data — contrast with the
//!    legacy lexer in `bnfgen-core`, which decoded strings and integers at
//!    lex time.
//! 2. **The token buffer is the source-preserving representation.**
//!    Whitespace and comments are real token kinds, not `skip` patterns, so
//!    the buffer covers ordinary input without gaps and token iteration can
//!    reconstruct what the user sees. Invalid input is the remaining gap in
//!    this slice: it is reported as a retained syntax error and dropped from
//!    the buffer until ticket 02 gives it token representation.

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
    /// decoded here. An unterminated string does not match and surfaces as
    /// lexical input the document retains (see `crate::lexer`).
    #[regex(r#""(\\["nrt\\]|[^"\\])*""#)]
    Str,
    /// Horizontal and vertical spacing. The character set matches the
    /// legacy skip pattern exactly (`\r` is deliberately absent there and
    /// stays absent here; ticket 02 revisits it when invalid input gains
    /// token representation).
    #[regex(r"[ \t\n\f]+")]
    Whitespace,
    /// A line comment. It extends to (not including) the newline, so a
    /// comment at end of file still lexes — the newline belongs to
    /// `Whitespace` when present. `allow_greedy` is safe here: the class
    /// cannot cross the `\n` that bounds it.
    #[regex(r"//[^\n]*", allow_greedy = true)]
    Comment,
}

impl TokenKind {
    /// Whether this kind is trivia: present in the token buffer for source
    /// fidelity, but never significant to grammar recognition.
    pub fn is_trivia(&self) -> bool {
        matches!(self, TokenKind::Whitespace | TokenKind::Comment)
    }
}

/// A token in a parsed document: a kind plus the UTF-8 byte range it
/// occupies in the retained source.
///
/// The token carries no text. Its raw lexeme is obtained by slicing the
/// document source with [`range`](Self::range) — one canonical copy of the
/// text, per ADR 0003.
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
