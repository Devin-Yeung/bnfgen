//! # bnfgen-syntax
//!
//! The Bnfgen source language: lexing, grammar recognition, recovery, and
//! the token-backed [`ParsedDocument`] that callers query.
//!
//! This crate is the sole owner of the Logos lexer and one private tolerant
//! recursive-descent parser.
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
//! [`SyntaxDiagnostic`] *data* inside the document, never returned as a failed
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
//! The parser records complete and partial syntax through the same typed
//! views. Diagnostics explain missing or unexpected syntax, while recovery
//! observations name real source text the parser could not structurally use.
//! A later slice adds cursor-context queries. The temporary strict lowering
//! onto legacy models stays outside this crate by design.

mod document;
mod error;
mod lexer;
mod parser;
mod records;
mod recovery;
mod token;
mod views;

pub use document::ParsedDocument;
pub use error::{ExpectedSyntax, SyntaxDiagnostic, SyntaxDiagnosticKind};
pub use recovery::{RecoveryKind, RecoveryObservation};
pub use token::{SyntaxToken, TokenKind};
pub use views::{
    AlternativeSyntax, IntegerSyntax, NonTerminalSyntax, RegexSyntax, RepeatSyntax, RuleSyntax,
    StringSyntax, StructuralState, SymbolKind, SymbolSyntax,
};

/// Parse source text into a [`ParsedDocument`].
///
/// Total for every input: a document is always returned, and anything the
/// lexer or parser could not accept is retained as diagnostics and recovery
/// observations. Recognized rules are observable through
/// [`ParsedDocument::rules`] regardless of trouble elsewhere.
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
