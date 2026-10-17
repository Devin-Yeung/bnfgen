//! Recovery never happens silently.
//!
//! Every source range the parser skipped or failed to attach must be covered
//! by a syntax diagnostic. Diagnostics are the single correctness signal, so
//! a caller that checks only diagnostics cannot miss a recovery.

// The snapshot helpers are unused here; only the fixture loader is needed.
#[allow(dead_code)]
mod common;

use std::ops::Range;

use bnfgen_syntax::{parse, ParsedDocument};

fn contains(outer: &Range<usize>, inner: &Range<usize>) -> bool {
    outer.start <= inner.start && inner.end <= outer.end
}

fn assert_recovery_is_diagnosed(doc: &ParsedDocument, label: &str) {
    let source_len = doc.source().len();
    let diagnostics: Vec<Range<usize>> = doc
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.range())
        .collect();

    for observation in doc.recovery() {
        let range = observation.range();
        assert!(
            range.start < range.end && range.end <= source_len,
            "{label}: recovery range {range:?} is empty or out of bounds"
        );
        assert!(
            diagnostics.iter().any(|diagnostic| contains(diagnostic, &range)),
            "{label}: recovery range {range:?} ({:?}) has no covering diagnostic; diagnostics: {diagnostics:?}",
            observation.kind(),
        );
    }
}

#[test]
fn every_recovery_observation_is_covered_by_a_diagnostic() {
    for (name, source) in common::fixtures() {
        assert_recovery_is_diagnosed(&parse(&source), &format!("fixture {name}"));
    }
}

#[test]
fn every_prefix_of_every_fixture_keeps_recovery_diagnosed() {
    for (name, source) in common::fixtures() {
        for end in source
            .char_indices()
            .map(|(index, _)| index)
            .chain(std::iter::once(source.len()))
        {
            let prefix = &source[..end];
            assert_recovery_is_diagnosed(&parse(prefix), &format!("fixture {name} prefix {end}"));
        }
    }
}
