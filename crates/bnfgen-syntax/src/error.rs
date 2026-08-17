//! Syntax errors as retained data.
//!
//! A parse never fails: when recognition goes wrong, the document keeps a
//! `SyntaxError` describing *what kind* of problem occurred *where*, so
//! callers can reason about errors structurally instead of parsing message
//! strings. Rendering (Miette or otherwise) stays outside this crate.

use std::ops::Range;

/// A syntax error attached to a [`ParsedDocument`](crate::ParsedDocument).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SyntaxError {
    kind: SyntaxErrorKind,
    range: Range<usize>,
}

impl SyntaxError {
    pub(super) fn new(kind: SyntaxErrorKind, range: Range<usize>) -> Self {
        Self { kind, range }
    }

    pub fn kind(&self) -> SyntaxErrorKind {
        self.kind
    }

    /// The UTF-8 byte range the error refers to in the retained source.
    /// An end-of-input error points at an empty range at that offset.
    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }
}

/// The structured kind of a syntax error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SyntaxErrorKind {
    /// Input that does not match any token rule. In this slice the
    /// offending text is dropped from the token buffer; ticket 02 retains
    /// it as recoverable token input instead.
    UnrecognizedInput,
    /// A well-formed token appeared where the grammar could not accept it.
    UnexpectedToken,
    /// The document ended where the grammar required more input.
    UnexpectedEof,
}
