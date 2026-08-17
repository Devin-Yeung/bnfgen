//! Normalized public snapshots of the parse-and-query seam (tickets 01–02).
//!
//! The renderer below is the spec's "normalized representation": public
//! token kinds and byte ranges plus the raw source slice, retained errors,
//! and recognized rule views — never a private `Debug` layout. The legacy
//! lexer's snapshot (decoded `Int(usize)` / `Str(String)` payloads) is what
//! this replaces; the raw spelling is asserted verbatim here.
//!
//! Ticket 03 extends the rule block with construct lines for the typed
//! forms it adds. Add a new case by dropping a `.bnfgen` file in
//! `tests/fixtures/` and listing it in `common::FIXTURES`.

mod common;

use std::fmt::Write;

use bnfgen_syntax::{parse, ParsedDocument, SymbolKind};
use insta::assert_snapshot;

/// Compile-time assertion that document snapshots can be retained and
/// queried across worker threads (spec: `ParsedDocument` is `Send + Sync`).
/// Instantiating `require` fails to build if the property regresses.
fn require<T: Send + Sync>() {}

#[test]
fn parsed_document_is_send_and_sync() {
    require::<ParsedDocument>();
}

/// Write `source` so a snapshot can be read as grammar text: real newlines
/// and quotes, with `\r` / `\t` / other controls escaped. Trailing newline
/// is kept as a real newline; its absence is a metadata line so EOF-shaped
/// fixtures stay unambiguous.
fn write_input(out: &mut String, source: &str) {
    out.push_str("input:\n");
    for ch in source.chars() {
        match ch {
            '\n' => out.push('\n'),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => write!(out, "{}", c.escape_default()).unwrap(),
            c => out.push(c),
        }
    }
    if !source.ends_with('\n') {
        out.push_str("\n[no trailing newline]\n");
    }
    out.push('\n');
}

/// Render a document as its normalized public snapshot: the fixture source
/// (readable, not Debug-quoted), then every token in source order with
/// kind, byte range, and raw slice; then errors; then recognized rules.
fn render(doc: &ParsedDocument) -> String {
    let mut out = String::new();
    write_input(&mut out, doc.source());
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
    out.push_str("rules:\n");
    for rule in doc.rules() {
        let range = rule.range();
        writeln!(
            &mut out,
            "{:03}..{:03} {}",
            range.start,
            range.end,
            rule.name().name(),
        )
        .unwrap();
        for alternative in rule.alternatives() {
            let range = alternative.range();
            writeln!(&mut out, "  alt {:03}..{:03}", range.start, range.end).unwrap();
            for symbol in alternative.symbols() {
                let range = symbol.range();
                match symbol.kind() {
                    SymbolKind::Terminal => writeln!(
                        &mut out,
                        "    {:03}..{:03} Terminal {:?}",
                        range.start,
                        range.end,
                        symbol.text(),
                    )
                    .unwrap(),
                    SymbolKind::NonTerminal => {
                        let name = symbol
                            .as_non_terminal()
                            .expect("NonTerminal kind has a non-terminal view")
                            .name();
                        writeln!(
                            &mut out,
                            "    {:03}..{:03} NonTerminal {}",
                            range.start, range.end, name,
                        )
                        .unwrap();
                    }
                }
            }
        }
    }
    out
}

#[test]
fn public_seam_fixtures_snapshot() {
    for (name, source) in common::FIXTURES {
        let doc = parse(source);
        common::assert_tiles_source(&doc);
        common::assert_token_lookup(&doc);
        insta::with_settings!({ omit_expression => true }, {
            assert_snapshot!(format!("fixture_{name}"), render(&doc));
        });
    }
}
