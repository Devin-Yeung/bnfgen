//! Private, byte-range-backed storage for recognized syntax facts.
//!
//! The parser records only facts established by source. These records are not
//! a generic concrete syntax tree: each refers to source ranges in the
//! document's complete token buffer. Public borrowing views in `views.rs`
//! hide this storage and expose structural state instead.

use std::ops::Range;

/// Structural state of a source-established syntax fact.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StructuralState {
    Complete,
    Partial,
}

#[derive(Debug)]
pub(crate) struct RuleRecord {
    pub(crate) span: Range<usize>,
    pub(crate) lhs: Option<NonTerminalRecord>,
    pub(crate) definition: Option<Range<usize>>,
    pub(crate) alts: Vec<AlternativeRecord>,
    pub(crate) terminator: Option<Range<usize>>,
    pub(crate) state: StructuralState,
    pub(crate) has_recovery: bool,
}

#[derive(Debug)]
pub(crate) struct AlternativeRecord {
    pub(crate) span: Range<usize>,
    pub(crate) weight: Option<Range<usize>>,
    pub(crate) symbols: Vec<SymbolRecord>,
    pub(crate) repeat: Option<RepeatRecord>,
    pub(crate) state: StructuralState,
    pub(crate) has_recovery: bool,
}

#[derive(Debug)]
pub(crate) struct NonTerminalRecord {
    pub(crate) span: Range<usize>,
    pub(crate) opening: Range<usize>,
    pub(crate) name: Option<Range<usize>>,
    pub(crate) type_separator: Option<Range<usize>>,
    pub(crate) ty: Option<StringRecord>,
    pub(crate) closing: Option<Range<usize>>,
    pub(crate) state: StructuralState,
}

#[derive(Debug)]
pub(crate) struct StringRecord {
    pub(crate) span: Range<usize>,
    pub(crate) terminated: bool,
}

#[derive(Debug)]
pub(crate) struct RepeatRecord {
    pub(crate) span: Range<usize>,
    pub(crate) opening: Range<usize>,
    pub(crate) lower: Option<Range<usize>>,
    pub(crate) comma: Option<Range<usize>>,
    pub(crate) upper: Option<Range<usize>>,
    pub(crate) closing: Option<Range<usize>>,
    pub(crate) state: StructuralState,
}

#[derive(Debug)]
pub(crate) struct RegexRecord {
    pub(crate) span: Range<usize>,
    pub(crate) opening_parenthesis: Option<Range<usize>>,
    pub(crate) pattern: Option<StringRecord>,
    pub(crate) closing_parenthesis: Option<Range<usize>>,
    pub(crate) state: StructuralState,
}

#[derive(Debug)]
pub(crate) enum SymbolRecord {
    Terminal(StringRecord),
    NonTerminal(NonTerminalRecord),
    Regex(RegexRecord),
}

impl SymbolRecord {
    pub(crate) fn state(&self) -> StructuralState {
        match self {
            Self::Terminal(record) => {
                if record.terminated {
                    StructuralState::Complete
                } else {
                    StructuralState::Partial
                }
            }
            Self::NonTerminal(record) => record.state,
            Self::Regex(record) => record.state,
        }
    }
}
