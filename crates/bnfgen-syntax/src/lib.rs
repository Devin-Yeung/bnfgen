//! # bnfgen-syntax
//!
//! The Bnfgen source language: lexing, grammar recognition, recovery, and
//! the token-backed [`ParsedDocument`] that callers query.
//!
//! This crate is the sole owner of the Logos lexer and the LALRPOP grammar.
//! It knows the *shape* of a grammar file and nothing about its
//! *meaning*: literal decoding, integer parsing, regular-expression
//! compilation, name resolution, and generation invariants all live
//! downstream. It has no dependency on Rowan, Miette, Petgraph, Rand, or
//! any generation, CLI, or LSP code; the storage stays token-backed so
//! recognized syntax can remain source-preserving and independently queried.
//!
//! ## The contract
//!
//! [`parse`] is **total**: every `&str` — empty, malformed, incomplete —
//! produces a `ParsedDocument`. Problems are retained as
//! [`SyntaxError`] *data* inside the document, never returned as a failed
//! parse, so a caller (a language server, say) never needs a "parsing
//! failed" state variant.
//!
//! A document owns the original source plus a complete, source-ordered
//! token buffer, and answers queries through borrowing, language-specific
//! views ([`RuleSyntax`] and friends). Two invariants hold throughout:
//!
//! - **Raw lexemes.** Token and view text is always the exact source
//!   spelling, sliced by UTF-8 byte range. Nothing is decoded here.
//! - **Private storage.** Token identifiers, parser-generated types, and
//!   the range-backed records behind the views never cross the crate
//!   interface, so recovery can improve without breaking callers.
//!
//! Line/column and UTF-16 positions are deliberately absent: they are an
//! adapter concern (LSP), not part of the language model.
//!
//! ## Current state
//!
//! This is the current migration slice of the syntax rewrite: every valid
//! language construct is recognized through typed views, and
//! the token buffer retains every byte of input — invalid input and
//! unterminated strings included — as recoverable token kinds. Broken rules
//! recover through LALRPOP's grammar-level error recovery, and recognized
//! facts inside incomplete rules remain available through the same typed
//! views. A later slice adds cursor-context queries.
//! The temporary strict lowering onto the legacy models stays outside this
//! crate by design.
//!
//! The governing documents are `.scratch/bnfgen-syntax-rewrite/spec.md` and
//! `docs/architecture/frontend.md`.

mod document;
mod error;
mod lexer;
mod records;
mod token;
mod views;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(parser);
lalrpop_mod!(partial_parser);

pub use document::ParsedDocument;
pub use error::{SyntaxError, SyntaxErrorKind};
pub use token::{SyntaxToken, TokenKind};
pub use views::{
    AlternativeSyntax, IntegerSyntax, NonTerminalSyntax, RegexSyntax, RepeatSyntax, RuleSyntax,
    StringSyntax, SymbolKind, SymbolSyntax,
};

/// Parse source text into a [`ParsedDocument`].
///
/// Total for every input: a document is always returned, and anything the
/// lexer or grammar could not accept is retained inside it as
/// [`SyntaxError`]s. Recognized rules are observable through
/// [`ParsedDocument::rules`] regardless of errors elsewhere.
pub fn parse(source: &str) -> ParsedDocument {
    document::parse(source)
}

// Compile-time property guard, not dead weight: the function body is
// type-checked even though nothing calls it, so the build fails the moment
// `ParsedDocument` stops being shareable across threads. The public-seam
// test in `tests/` asserts the same property from outside the crate.
#[allow(dead_code)]
fn _assert_parsed_document_send_sync() {
    fn require<T: Send + Sync>() {}
    require::<ParsedDocument>();
}
