//! Normalized public snapshots of the parse-and-query seam.
//!
//! [`common::snapshot`] is the spec's "normalized representation": an
//! eza-style Unicode tree of public views — never a private `Debug` layout.
//! Add a new case by dropping a `.bnfgen` file in `tests/fixtures/`.

mod common;

use bnfgen_syntax::ParsedDocument;
use insta::assert_snapshot;

/// Compile-time assertion that document snapshots can be retained and
/// queried across worker threads (spec: `ParsedDocument` is `Send + Sync`).
/// Instantiating `require` fails to build if the property regresses.
fn require<T: Send + Sync>() {}

#[test]
fn parsed_document_is_send_and_sync() {
    require::<ParsedDocument>();
}

#[test]
fn public_seam_fixtures_snapshot() {
    for (name, source) in common::fixtures() {
        let doc = bnfgen_syntax::parse(&source);
        insta::with_settings!({ omit_expression => true }, {
            assert_snapshot!(format!("fixture_{name}"), common::snapshot(&doc));
        });
    }
}
