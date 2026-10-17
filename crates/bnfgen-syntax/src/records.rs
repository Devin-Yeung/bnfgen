//! Private, range-backed syntax model: the single source of truth for a
//! parsed document's structure.
//!
//! Every record holds only what source established. Required children that
//! were never written are `None`, and source consumed by recovery without
//! structural attachment is kept in the owning record's `skipped` list. The
//! recovery observations are derived from these records; nothing is stored
//! twice.

use std::ops::Range;

use crate::recovery::{RecoveryKind, RecoveryObservation};

/// Source the parser consumed without attaching it to any recognized child.
#[derive(Debug, Clone)]
pub(crate) struct Skipped {
    pub(crate) range: Range<usize>,
    pub(crate) kind: RecoveryKind,
}

impl Skipped {
    fn observation(&self) -> RecoveryObservation {
        RecoveryObservation::new(self.range.clone(), self.kind)
    }
}

/// The whole document: its rules and the source between them that no rule
/// claimed.
#[derive(Debug)]
pub(crate) struct SourceFile {
    pub(crate) rules: Vec<RuleRecord>,
    pub(crate) skipped: Vec<Skipped>,
}

impl SourceFile {
    /// Every recovery observation in the document, in source order.
    pub(crate) fn recovery(&self) -> Vec<RecoveryObservation> {
        let mut out: Vec<RecoveryObservation> =
            self.skipped.iter().map(Skipped::observation).collect();
        for rule in &self.rules {
            rule.collect_recovery(&mut out);
        }
        out.sort_by_key(|observation| observation.range().start);
        out
    }
}

#[derive(Debug)]
pub(crate) struct RuleRecord {
    pub(crate) span: Range<usize>,
    pub(crate) lhs: NonTerminalRecord,
    pub(crate) definition: Option<Range<usize>>,
    pub(crate) alts: Vec<AlternativeRecord>,
    pub(crate) terminator: Option<Range<usize>>,
    pub(crate) skipped: Vec<Skipped>,
}

impl RuleRecord {
    fn collect_recovery(&self, out: &mut Vec<RecoveryObservation>) {
        out.extend(self.skipped.iter().map(Skipped::observation));
        self.lhs.collect_recovery(out);
        for alt in &self.alts {
            alt.collect_recovery(out);
        }
    }
}

#[derive(Debug)]
pub(crate) struct AlternativeRecord {
    pub(crate) span: Range<usize>,
    pub(crate) weight: Option<Range<usize>>,
    pub(crate) symbols: Vec<SymbolRecord>,
    pub(crate) repeat: Option<RepeatRecord>,
}

impl AlternativeRecord {
    fn collect_recovery(&self, out: &mut Vec<RecoveryObservation>) {
        for symbol in &self.symbols {
            symbol.collect_recovery(out);
        }
    }
}

#[derive(Debug)]
pub(crate) struct NonTerminalRecord {
    pub(crate) span: Range<usize>,
    pub(crate) opening: Range<usize>,
    pub(crate) name: Option<Range<usize>>,
    pub(crate) type_separator: Option<Range<usize>>,
    pub(crate) ty: Option<StringRecord>,
    pub(crate) closing: Option<Range<usize>>,
}

impl NonTerminalRecord {
    fn collect_recovery(&self, out: &mut Vec<RecoveryObservation>) {
        if let Some(ty) = &self.ty {
            ty.collect_recovery(out);
        }
    }
}

#[derive(Debug)]
pub(crate) struct StringRecord {
    pub(crate) span: Range<usize>,
    pub(crate) terminated: bool,
}

impl StringRecord {
    fn collect_recovery(&self, out: &mut Vec<RecoveryObservation>) {
        if !self.terminated {
            out.push(RecoveryObservation::new(
                self.span.clone(),
                RecoveryKind::UnterminatedString,
            ));
        }
    }
}

#[derive(Debug)]
pub(crate) struct RepeatRecord {
    pub(crate) span: Range<usize>,
    pub(crate) opening: Range<usize>,
    pub(crate) lower: Option<Range<usize>>,
    pub(crate) comma: Option<Range<usize>>,
    pub(crate) upper: Option<Range<usize>>,
    pub(crate) closing: Option<Range<usize>>,
}

impl RepeatRecord {}

#[derive(Debug)]
pub(crate) struct RegexRecord {
    pub(crate) span: Range<usize>,
    pub(crate) opening_parenthesis: Option<Range<usize>>,
    pub(crate) pattern: Option<StringRecord>,
    pub(crate) closing_parenthesis: Option<Range<usize>>,
}

impl RegexRecord {
    fn collect_recovery(&self, out: &mut Vec<RecoveryObservation>) {
        if let Some(pattern) = &self.pattern {
            pattern.collect_recovery(out);
        }
    }
}

#[derive(Debug)]
pub(crate) enum SymbolRecord {
    Terminal(StringRecord),
    NonTerminal(NonTerminalRecord),
    Regex(RegexRecord),
}

impl SymbolRecord {
    fn collect_recovery(&self, out: &mut Vec<RecoveryObservation>) {
        match self {
            Self::Terminal(record) => record.collect_recovery(out),
            Self::NonTerminal(record) => record.collect_recovery(out),
            Self::Regex(record) => record.collect_recovery(out),
        }
    }
}
