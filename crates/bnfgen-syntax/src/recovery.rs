//! Recovery observations for source text that could not be structurally used.

use std::ops::Range;

/// An observed source range affected by lexing or parser recovery.
///
/// This is not a diagnostic: diagnostics explain what is wrong, whereas a
/// recovery observation identifies actual source text that was retained but
/// not attached to recognized syntax while parsing continued.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryObservation {
    range: Range<usize>,
    kind: RecoveryKind,
}

impl RecoveryObservation {
    pub(crate) fn new(range: Range<usize>, kind: RecoveryKind) -> Self {
        debug_assert!(range.start < range.end);
        Self { range, kind }
    }

    pub fn range(&self) -> Range<usize> {
        self.range.clone()
    }

    pub fn kind(&self) -> RecoveryKind {
        self.kind
    }
}

/// Why a source range was retained as a recovery observation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecoveryKind {
    InvalidLexeme,
    UnterminatedString,
    UnexpectedToken,
}
