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

use bnfgen_syntax::{
    parse, NonTerminalSyntax, ParsedDocument, StructuralState, SymbolKind, SyntaxToken,
};
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

    match (non_terminal.name_range(), non_terminal.name()) {
        (Some(name_range), Some(name)) => {
            tree.push(Tree::new(format!(
                "name {} {}",
                range_label(name_range.start, name_range.end),
                name,
            )));
        }
        (None, None) => {
            tree.push(Tree::new("name <missing>".to_owned()));
        }
        _ => unreachable!("name range and text come from the same optional record"),
    }
    if let Some(separator) = non_terminal.type_separator_range() {
        tree.push(Tree::new(format!(
            "type-separator {}",
            range_label(separator.start, separator.end),
        )));
    }
    if let Some(ty) = non_terminal.ty() {
        let ty_range = ty.range();
        tree.push(Tree::new(format!(
            "type {} {}{}",
            range_label(ty_range.start, ty_range.end),
            ty.text(),
            if ty.is_terminated() {
                ""
            } else {
                " <unterminated>"
            },
        )));
    }
    if let Some(closing) = non_terminal.closing_delimiter_range() {
        tree.push(Tree::new(format!(
            "closing {}",
            range_label(closing.start, closing.end),
        )));
    } else {
        tree.push(Tree::new("closing <missing>".to_owned()));
    }
    tree
}

fn structural_state_label(state: StructuralState) -> &'static str {
    match state {
        StructuralState::Complete => "complete",
        StructuralState::Partial => "partial",
    }
}

/// Public-seam tree: tokens, diagnostics, recovery observations, and rules
/// are siblings under `document`. Trivia (whitespace, comments) is a token
/// leaf, not a CST node — there is no generic tree; comments never nest
/// under rules.
fn document_tree(doc: &ParsedDocument) -> Tree<String> {
    let tokens = Tree::new("tokens".to_owned())
        .with_leaves(doc.tokens().map(|token| token_leaf(doc, token)));

    let diagnostics = Tree::new("diagnostics".to_owned()).with_leaves(
        doc.diagnostics().iter().map(|diagnostic| {
            let range = diagnostic.range();
            Tree::new(format!(
                "{} {:?}",
                range_label(range.start, range.end),
                diagnostic.kind()
            ))
        }),
    );

    let recovery =
        Tree::new("recovery".to_owned()).with_leaves(doc.recovery().map(|observation| {
            let range = observation.range();
            Tree::new(format!(
                "{} {:?}{}",
                range_label(range.start, range.end),
                observation.kind(),
                inline_source(doc.slice(range)),
            ))
        }));

    let rules = Tree::new("rules".to_owned()).with_leaves(doc.rules().map(|rule| {
        let range = rule.range();
        let name = rule.lhs().and_then(|lhs| lhs.name()).unwrap_or("<missing>");
        let mut rule_tree = Tree::new(format!(
            "{} {} [{}{}]",
            range_label(range.start, range.end),
            name,
            structural_state_label(rule.structural_state()),
            if rule.has_recovery() {
                ", recovered"
            } else {
                ""
            },
        ));
        if let Some(lhs) = rule.lhs() {
            rule_tree.push(non_terminal_tree("lhs", lhs));
        } else {
            rule_tree.push(Tree::new("lhs <missing>".to_owned()));
        }
        if let Some(definition) = rule.definition_range() {
            rule_tree.push(Tree::new(format!(
                "definition {}",
                range_label(definition.start, definition.end),
            )));
        } else {
            rule_tree.push(Tree::new("definition <missing>".to_owned()));
        }
        rule_tree.extend(rule.alternatives().map(|alternative| {
            let range = alternative.range();
            let mut alternative_tree = Tree::new(format!(
                "alt {} [{}{}]",
                range_label(range.start, range.end),
                structural_state_label(alternative.structural_state()),
                if alternative.has_recovery() {
                    ", recovered"
                } else {
                    ""
                },
            ));
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
                            "{} Terminal{}{}",
                            range_label(range.start, range.end),
                            inline_source(terminal.text()),
                            if terminal.is_terminated() {
                                ""
                            } else {
                                " <unterminated>"
                            },
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
                        let mut regex_tree = Tree::new(format!(
                            "{} Regex {}",
                            range_label(range.start, range.end),
                            regex.text(),
                        ));
                        if let Some(opening) = regex.opening_parenthesis_range() {
                            regex_tree.push(Tree::new(format!(
                                "opening-parenthesis {}",
                                range_label(opening.start, opening.end),
                            )));
                        } else {
                            regex_tree.push(Tree::new("opening-parenthesis <missing>".to_owned()));
                        }
                        if let Some(pattern) = regex.pattern() {
                            let pattern_range = pattern.range();
                            regex_tree.push(Tree::new(format!(
                                "pattern {} {}{}",
                                range_label(pattern_range.start, pattern_range.end),
                                pattern.text(),
                                if pattern.is_terminated() {
                                    ""
                                } else {
                                    " <unterminated>"
                                },
                            )));
                        } else {
                            regex_tree.push(Tree::new("pattern <missing>".to_owned()));
                        }
                        if let Some(closing) = regex.closing_parenthesis_range() {
                            regex_tree.push(Tree::new(format!(
                                "closing-parenthesis {}",
                                range_label(closing.start, closing.end),
                            )));
                        } else {
                            regex_tree.push(Tree::new("closing-parenthesis <missing>".to_owned()));
                        }
                        regex_tree
                    }
                }
            }));
            if let Some(repeat) = alternative.repeat() {
                let range = repeat.range();
                let mut repeat_tree = Tree::new(format!(
                    "repeat {} {}",
                    range_label(range.start, range.end),
                    repeat.text(),
                ));
                if let Some(lower) = repeat.lower_bound() {
                    let lower_range = lower.range();
                    repeat_tree.push(Tree::new(format!(
                        "lower {} {}",
                        range_label(lower_range.start, lower_range.end),
                        lower.text(),
                    )));
                } else {
                    repeat_tree.push(Tree::new("lower <missing>".to_owned()));
                }
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
                if let Some(closing) = repeat.closing_delimiter_range() {
                    repeat_tree.push(Tree::new(format!(
                        "closing {}",
                        range_label(closing.start, closing.end),
                    )));
                } else {
                    repeat_tree.push(Tree::new("closing <missing>".to_owned()));
                }
                alternative_tree.push(repeat_tree);
            }
            alternative_tree
        }));
        if let Some(terminator) = rule.terminator_range() {
            rule_tree.push(Tree::new(format!(
                "terminator {}",
                range_label(terminator.start, terminator.end),
            )));
        } else {
            rule_tree.push(Tree::new("terminator <missing>".to_owned()));
        }
        rule_tree
    }));

    Tree::new("document".to_owned()).with_leaves([tokens, diagnostics, recovery, rules])
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
