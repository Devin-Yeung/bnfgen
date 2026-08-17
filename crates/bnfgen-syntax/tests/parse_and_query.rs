//! Ticket 01 acceptance tests: the public parse-and-query seam.
//!
//! Everything here goes through `bnfgen_syntax::parse` and `ParsedDocument`
//! queries only — no private storage, token identifiers, or parser
//! machinery. Tests describe caller-visible behavior (the spec's testing
//! contract), so internal record changes cannot churn them.

use bnfgen_syntax::{parse, ParsedDocument, RuleSyntax, SymbolKind, SyntaxErrorKind, TokenKind};

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

/// The ticket's supported complete-rule fixture: one untyped rule with two
/// alternatives of plain symbols.
const COMPLETE_RULE: &str = r#"<greeting> ::= "hello" <name> | "bye";"#;

#[test]
fn complete_rule_yields_source_ordered_tokens() {
    let doc = parse(COMPLETE_RULE);

    // Trivia is part of the buffer: the sequence covers the fixture
    // exactly, whitespace included, so iteration reconstructs the input.
    let kinds: Vec<_> = doc.tokens().map(|token| token.kind()).collect();
    assert_eq!(
        kinds,
        vec![
            TokenKind::LAngle,
            TokenKind::Id,
            TokenKind::RAngle,
            TokenKind::Whitespace,
            TokenKind::Def,
            TokenKind::Whitespace,
            TokenKind::Str,
            TokenKind::Whitespace,
            TokenKind::LAngle,
            TokenKind::Id,
            TokenKind::RAngle,
            TokenKind::Whitespace,
            TokenKind::Or,
            TokenKind::Whitespace,
            TokenKind::Str,
            TokenKind::Semi,
        ]
    );

    // Ranges are source-ordered and, absent invalid input, tile the source.
    let tokens: Vec<_> = doc.tokens().collect();
    assert_eq!(tokens.first().unwrap().range().start, 0);
    assert_eq!(tokens.last().unwrap().range().end, COMPLETE_RULE.len());
    for pair in tokens.windows(2) {
        assert_eq!(
            pair[0].range().end,
            pair[1].range().start,
            "tokens {:?} {:?} do not tile the source",
            pair[0],
            pair[1],
        );
    }

    // Raw token text comes from slicing the retained source, quotes kept.
    let str_token = tokens
        .iter()
        .find(|token| token.kind() == TokenKind::Str)
        .expect("fixture contains a string literal");
    assert_eq!(&doc.source()[str_token.range()], r#""hello""#);
}

#[test]
fn complete_rule_is_queryable_through_typed_views() {
    let doc = parse(COMPLETE_RULE);
    assert!(doc.errors().is_empty(), "fixture must parse without errors");

    let rules: Vec<RuleSyntax<'_>> = doc.rules().collect();
    assert_eq!(rules.len(), 1);
    let rule = &rules[0];
    assert_eq!(rule.range(), 0..COMPLETE_RULE.len());
    assert_eq!(rule.text(), COMPLETE_RULE);

    // Left-hand side keeps its bracketed form and bare spelling distinct.
    let name = rule.name();
    assert_eq!(name.text(), "<greeting>");
    assert_eq!(name.name(), "greeting");
    assert_eq!(
        name.name_range(),
        COMPLETE_RULE.find("greeting").unwrap()..COMPLETE_RULE.find('>').unwrap(),
    );

    let alternatives: Vec<_> = rule.alternatives().collect();
    assert_eq!(alternatives.len(), 2);

    let first = &alternatives[0];
    assert_eq!(first.text(), r#""hello" <name>"#);
    let symbols: Vec<_> = first.symbols().collect();
    assert_eq!(symbols.len(), 2);
    assert_eq!(symbols[0].kind(), SymbolKind::Terminal);
    assert_eq!(symbols[0].text(), r#""hello""#); // raw: undecoded, quoted
    assert!(symbols[0].as_non_terminal().is_none());
    assert_eq!(symbols[1].kind(), SymbolKind::NonTerminal);
    let reference = symbols[1].as_non_terminal().unwrap();
    assert_eq!(reference.name(), "name");
    assert_eq!(reference.text(), "<name>");

    let second = &alternatives[1];
    assert_eq!(second.text(), r#""bye""#);
    assert_eq!(second.symbols().count(), 1);
}

#[test]
fn syntactically_malformed_input_still_returns_a_document() {
    // Missing the rule terminator: lexes cleanly, grammar recognition
    // fails at end of input.
    let source = r#"<greeting> ::= "hello""#;
    let doc = parse(source);

    assert_eq!(doc.source(), source);
    assert_eq!(doc.rules().count(), 0);

    let errors = doc.errors();
    assert_eq!(errors.len(), 1);
    assert_eq!(errors[0].kind(), SyntaxErrorKind::UnexpectedEof);
    let range = errors[0].range();
    assert!(range.start <= range.end && range.end <= source.len());

    // The token buffer survives the parse failure.
    assert!(doc.tokens().count() > 0);
}

#[test]
fn lexically_invalid_input_is_reported_but_not_fatal() {
    // The stray `@` matches no token rule, yet the complete rule before it
    // is still recognized: lexical failure never terminates the document.
    let source = r#"<greeting> ::= "hello"; @"#;
    let doc = parse(source);

    assert_eq!(doc.source(), source);
    assert_eq!(doc.rules().count(), 1);
    assert_eq!(
        doc.errors()[0].kind(),
        SyntaxErrorKind::UnrecognizedInput,
        "the invalid character must be retained as an error: {:?}",
        doc.errors(),
    );
}
