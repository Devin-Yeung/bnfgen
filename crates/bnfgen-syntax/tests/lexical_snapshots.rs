//! Normalized public snapshots of the lexical seam (ticket 02).
//!
//! The renderer below is the spec's "normalized representation": public
//! token kinds and byte ranges plus the raw source slice, one token per
//! line — never a private `Debug` layout. The legacy lexer's snapshot
//! (decoded `Int(usize)` / `Str(String)` payloads) is what this replaces;
//! the raw spelling is asserted verbatim here.

mod common;

use std::fmt::Write;

use bnfgen_syntax::parse;
use insta::assert_snapshot;

/// Render a document as its normalized public snapshot: every token in
/// source order with kind, byte range, and raw slice; then errors; then a
/// recognition count. Ticket 03 extends this with construct lines for the
/// typed forms it adds.
fn render(doc: &bnfgen_syntax::ParsedDocument) -> String {
    let mut out = String::new();
    for token in doc.tokens() {
        let range = token.range();
        let text = doc.slice(range.clone());
        writeln!(
            &mut out,
            "{:03}..{:03} {:?} {:?}",
            range.start,
            range.end,
            token.kind(),
            text,
        )
        .unwrap();
    }
    out.push_str("errors:\n");
    for error in doc.errors() {
        let range = error.range();
        writeln!(
            &mut out,
            "{:03}..{:03} {:?}",
            range.start,
            range.end,
            error.kind()
        )
        .unwrap();
    }
    writeln!(&mut out, "rules: {}", doc.rules().count()).unwrap();
    out
}

#[test]
fn lexical_fixtures_snapshot() {
    for (name, source) in common::FIXTURES {
        let doc = parse(source);
        common::assert_tiles_source(&doc);
        assert_snapshot!(format!("fixture_{name}"), render(&doc));
    }
}
