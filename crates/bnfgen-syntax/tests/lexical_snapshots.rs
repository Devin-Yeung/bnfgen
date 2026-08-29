//! Normalized public snapshots of the parse-and-query seam.
//!
//! The renderer below is the spec's "normalized representation": an
//! eza-style Unicode tree (via `termtree`) of public token kinds and byte
//! ranges plus the raw source slice, retained errors, and recognized rule
//! views — never a private `Debug` layout. The legacy lexer's snapshot
//! (decoded `Int(usize)` / `Str(String)` payloads) is what this replaces;
//! the raw spelling is asserted verbatim here.
//!
//! The rule block records the raw, ranged forms of every valid construct.
//! Add a new case by dropping a `.bnfgen` file in `tests/fixtures/`.

mod common;

use bnfgen_syntax::{parse, NonTerminalSyntax, ParsedDocument, SymbolKind, SyntaxToken};
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

/// Source text for a one-line tree label: omitted when it would inject a
/// line break. LF/CR remain visible in the `<input>` dump.
fn inline_source(text: &str) -> String {
    if text.contains(['\n', '\r']) {
        String::new()
    } else {
        format!(" {text}")
    }
}

fn token_leaf(doc: &ParsedDocument, token: SyntaxToken) -> Tree<String> {
    let range = token.range();
    Tree::new(format!(
        "{} {:?}{}",
        range_label(range.start, range.end),
        token.kind(),
        inline_source(doc.slice(range)),
    ))
}

/// Render the syntax that makes a non-terminal typed or untyped. Keeping
/// name and type ranges separate proves that callers never need to scan the
/// bracketed text to recover either fact.
fn non_terminal_tree(label: &str, non_terminal: NonTerminalSyntax<'_>) -> Tree<String> {
    let range = non_terminal.range();
    let mut tree = Tree::new(format!(
        "{} {label} {}",
        range_label(range.start, range.end),
        non_terminal.text(),
    ));

    let name_range = non_terminal.name_range();
    tree.push(Tree::new(format!(
        "name {} {}",
        range_label(name_range.start, name_range.end),
        non_terminal.name(),
    )));
    if let Some(ty) = non_terminal.ty() {
        let ty_range = ty.range();
        tree.push(Tree::new(format!(
            "type {} {}",
            range_label(ty_range.start, ty_range.end),
            ty.text(),
        )));
    }
    tree
}

/// Public-seam tree: tokens, errors, recovery, and rules are siblings under
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

    let recovery =
        Tree::new("recovery".to_owned()).with_leaves(doc.recovery_ranges().map(|range| {
            Tree::new(format!(
                "{}{}",
                range_label(range.start, range.end),
                inline_source(doc.slice(range)),
            ))
        }));

    let rules = Tree::new("rules".to_owned()).with_leaves(doc.rules().map(|rule| {
        let range = rule.range();
        let mut rule_tree = Tree::new(format!(
            "{} {}",
            range_label(range.start, range.end),
            rule.name().name(),
        ));
        rule_tree.push(non_terminal_tree("lhs", rule.name()));
        rule_tree.extend(rule.alternatives().map(|alternative| {
            let range = alternative.range();
            let mut alternative_tree =
                Tree::new(format!("alt {}", range_label(range.start, range.end)));
            if let Some(weight) = alternative.weight() {
                let range = weight.range();
                alternative_tree.push(Tree::new(format!(
                    "weight {} {}",
                    range_label(range.start, range.end),
                    weight.text(),
                )));
            }
            alternative_tree.extend(alternative.symbols().map(|symbol| {
                let range = symbol.range();
                match symbol.kind() {
                    SymbolKind::Terminal => {
                        let terminal = symbol
                            .as_terminal()
                            .expect("Terminal kind has a string-literal view");
                        Tree::new(format!(
                            "{} Terminal{}",
                            range_label(range.start, range.end),
                            inline_source(terminal.text()),
                        ))
                    }
                    SymbolKind::NonTerminal => non_terminal_tree(
                        "NonTerminal",
                        symbol
                            .as_non_terminal()
                            .expect("NonTerminal kind has a non-terminal view"),
                    ),
                    SymbolKind::Regex => {
                        let regex = symbol
                            .as_regex()
                            .expect("Regex kind has a regular-expression view");
                        let pattern = regex.pattern();
                        let pattern_range = pattern.range();
                        Tree::new(format!(
                            "{} Regex {}",
                            range_label(range.start, range.end),
                            regex.text(),
                        ))
                        .with_leaves([Tree::new(format!(
                            "pattern {} {}",
                            range_label(pattern_range.start, pattern_range.end),
                            pattern.text(),
                        ))])
                    }
                }
            }));
            if let Some(repeat) = alternative.repeat() {
                let range = repeat.range();
                let lower = repeat.lower_bound();
                let lower_range = lower.range();
                let mut repeat_tree = Tree::new(format!(
                    "repeat {} {}",
                    range_label(range.start, range.end),
                    repeat.text(),
                ))
                .with_leaves([Tree::new(format!(
                    "lower {} {}",
                    range_label(lower_range.start, lower_range.end),
                    lower.text(),
                ))]);
                if let Some(comma) = repeat.comma_range() {
                    repeat_tree.push(Tree::new(format!(
                        "comma {}",
                        range_label(comma.start, comma.end),
                    )));
                }
                if let Some(upper) = repeat.upper_bound() {
                    let range = upper.range();
                    repeat_tree.push(Tree::new(format!(
                        "upper {} {}",
                        range_label(range.start, range.end),
                        upper.text(),
                    )));
                }
                alternative_tree.push(repeat_tree);
            }
            alternative_tree
        }));
        rule_tree
    }));

    Tree::new("document".to_owned()).with_leaves([tokens, errors, recovery, rules])
}

/// Fixture source as written, then the public-seam tree. Slices in the
/// tree are the same raw spelling; nothing is Debug-quoted.
fn render(doc: &ParsedDocument) -> String {
    format!("<input>\n{}</input>\n{}", doc.source(), document_tree(doc))
}

#[test]
fn public_seam_fixtures_snapshot() {
    for (name, source) in common::fixtures() {
        let doc = parse(&source);
        insta::with_settings!({ omit_expression => true }, {
            assert_snapshot!(format!("fixture_{name}"), render(&doc));
        });
    }
}
