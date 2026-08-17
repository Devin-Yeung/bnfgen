//! Ticket 01 smoke tests: parse is total and the public query seam works.
//!
//! Token sequences, ranges, and typed-view details belong in later
//! snapshot / regression coverage. These tests only check that empty,
//! complete, and broken input all return a document you can query.

use bnfgen_syntax::{parse, ParsedDocument};

/// Compile-time assertion that document snapshots can be retained and
/// queried across worker threads (spec: `ParsedDocument` is `Send + Sync`).
/// Instantiating `require` fails to build if the property regresses.
fn require<T: Send + Sync>() {}

#[test]
fn parsed_document_is_send_and_sync() {
    require::<ParsedDocument>();
}

#[test]
fn empty_input_parses_to_an_empty_document() {
    let doc = parse("");
    assert_eq!(doc.source(), "");
    assert_eq!(doc.tokens().count(), 0);
    assert_eq!(doc.rules().count(), 0);
    assert!(doc.errors().is_empty());
}

#[test]
fn complete_rule_is_queryable() {
    let source = r#"<greeting> ::= "hello" <name> | "bye";"#;
    let doc = parse(source);
    assert_eq!(doc.source(), source);
    assert!(doc.errors().is_empty());
    assert!(doc.tokens().count() > 0);

    let rules: Vec<_> = doc.rules().collect();
    assert_eq!(rules.len(), 1);
    assert_eq!(rules[0].name().name(), "greeting");
    assert_eq!(rules[0].alternatives().count(), 2);
}

#[test]
fn syntactically_malformed_input_still_returns_a_document() {
    let source = r#"<greeting> ::= "hello""#;
    let doc = parse(source);
    assert_eq!(doc.source(), source);
    assert_eq!(doc.rules().count(), 0);
    assert!(!doc.errors().is_empty());
    assert!(doc.tokens().count() > 0);
}

#[test]
fn lexically_invalid_input_is_not_fatal() {
    let source = r#"<greeting> ::= "hello"; @"#;
    let doc = parse(source);
    assert_eq!(doc.source(), source);
    assert_eq!(doc.rules().count(), 1);
    assert!(!doc.errors().is_empty());
}
