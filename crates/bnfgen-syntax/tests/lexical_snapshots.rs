//! Normalized public snapshots of the parse-and-query seam (tickets 01–02).
//!
//! The renderer below is the spec's "normalized representation": an
//! eza-style Unicode tree (via `termtree`) of public token kinds and byte
//! ranges plus the raw source slice, retained errors, and recognized rule
//! views — never a private `Debug` layout. The legacy lexer's snapshot
//! (decoded `Int(usize)` / `Str(String)` payloads) is what this replaces;
//! the raw spelling is asserted verbatim here.
//!
//! Ticket 03 extends the rule block with construct lines for the typed
//! forms it adds. Add a new case by dropping a `.bnfgen` file in
//! `tests/fixtures/` and listing it in `common::FIXTURES`.

mod common;

use bnfgen_syntax::{parse, ParsedDocument, SymbolKind, SyntaxToken};
use insta::assert_snapshot;
use termtree::Tree;

/// Compile-time assertion that document snapshots can be retained and
/// queried across worker threads (spec: `ParsedDocument` is `Send + Sync`).
/// Instantiating `require` fails to build if the property regresses.
fn require<T: Send + Sync>() {}

#[test]
fn parsed_document_is_send_and_sync() {
    require::<ParsedDocument>();
}

fn range_label(start: usize, end: usize) -> String {
    format!("{start:03}..{end:03}")
}

fn token_leaf(doc: &ParsedDocument, token: SyntaxToken) -> Tree<String> {
    let range = token.range();
    Tree::new(format!(
        "{} {:?} {:?}",
        range_label(range.start, range.end),
        token.kind(),
        doc.slice(range),
    ))
}

/// Public-seam tree: `tokens`, `errors`, and `rules` as siblings under
/// `document`. Trivia (whitespace, comments) is a token leaf, not a CST
/// node — there is no generic tree; comments never nest under rules.
fn document_tree(doc: &ParsedDocument) -> Tree<String> {
    let tokens = Tree::new("tokens".to_owned())
        .with_leaves(doc.tokens().map(|token| token_leaf(doc, token)));

    let errors = Tree::new("errors".to_owned()).with_leaves(doc.errors().iter().map(|error| {
        let range = error.range();
        Tree::new(format!(
            "{} {:?}",
            range_label(range.start, range.end),
            error.kind()
        ))
    }));

    let rules = Tree::new("rules".to_owned()).with_leaves(doc.rules().map(|rule| {
        let range = rule.range();
        Tree::new(format!(
            "{} {}",
            range_label(range.start, range.end),
            rule.name().name(),
        ))
        .with_leaves(rule.alternatives().map(|alternative| {
            let range = alternative.range();
            Tree::new(format!("alt {}", range_label(range.start, range.end))).with_leaves(
                alternative.symbols().map(|symbol| {
                    let range = symbol.range();
                    let label = match symbol.kind() {
                        SymbolKind::Terminal => format!(
                            "{} Terminal {:?}",
                            range_label(range.start, range.end),
                            symbol.text(),
                        ),
                        SymbolKind::NonTerminal => {
                            let name = symbol
                                .as_non_terminal()
                                .expect("NonTerminal kind has a non-terminal view")
                                .name();
                            format!("{} NonTerminal {name}", range_label(range.start, range.end),)
                        }
                    };
                    Tree::new(label)
                }),
            )
        }))
    }));

    Tree::new("document".to_owned()).with_leaves([tokens, errors, rules])
}

/// Fixture source as raw text (quotes and newlines unescaped), then the
/// public-seam tree. Bytes between the tags match `doc.source()` except
/// that CR is written as the two-character sequence `\r` — insta (and
/// git) cannot retain a raw CR in a `.snap` file. Empty input is
/// `<input></input>` so the wrapper does not inject a blank line. A
/// missing trailing newline leaves `</input>` on the same line as the
/// last source character.
fn render(doc: &ParsedDocument) -> String {
    let source = doc.source().replace('\r', "\\r");
    if source.is_empty() {
        format!("<input></input>\n{}", document_tree(doc))
    } else {
        format!("<input>\n{source}</input>\n{}", document_tree(doc))
    }
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
