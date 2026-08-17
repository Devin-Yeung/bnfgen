//! Raw-lexeme regression tests (ticket 02).
//!
//! Plain asserts rather than snapshots: these pin lexical facts (kinds,
//! raw slices, absence of lexical errors) for significant kinds the
//! ticket-01 grammar does not map yet (`Int`, `Re`), whose grammar errors
//! will change in ticket 03 — the lexical contract must not.

use bnfgen_syntax::{parse, SyntaxErrorKind, TokenKind};

/// True when the document retained a lexical failure error, as opposed to
/// grammar-side errors.
fn has_lexical_error(doc: &bnfgen_syntax::ParsedDocument) -> bool {
    doc.errors().iter().any(|error| {
        matches!(
            error.kind(),
            SyntaxErrorKind::UnrecognizedInput | SyntaxErrorKind::UnterminatedString
        )
    })
}

#[test]
fn integer_overflow_is_tokenized_raw() {
    // 2^64: the legacy lexer parsed at lex time and rejected this with
    // `InvalidInteger`. Here the digits are just a raw Int lexeme; whether
    // the value fits is a downstream concern.
    let doc = parse(r#"<a> ::= "x" 18446744073709551616;"#);
    let overflow = doc
        .tokens()
        .find(|token| token.kind() == TokenKind::Int)
        .expect("the overflow digits must lex as an Int token");
    assert_eq!(doc.slice(overflow.range()), "18446744073709551616");
    assert!(!has_lexical_error(&doc));
}

#[test]
fn string_escapes_stay_undecoded() {
    let doc = parse(r#"<a> ::= "line \"quoted\" \n end";"#);
    let string = doc
        .tokens()
        .find(|token| token.kind() == TokenKind::Str)
        .expect("a well-formed string literal must lex");
    // Quotes retained, escapes as typed: nothing was decoded.
    assert_eq!(doc.slice(string.range()), r#""line \"quoted\" \n end""#);
    assert!(!has_lexical_error(&doc));
}

#[test]
fn re_body_stays_raw() {
    // The regular-expression body is an ordinary raw string; the syntax
    // crate neither compiles nor validates it.
    let doc = parse(r#"re "[a-z]+";"#);
    assert!(doc.tokens().any(|token| token.kind() == TokenKind::Re));
    let body = doc
        .tokens()
        .find(|token| token.kind() == TokenKind::Str)
        .expect("the re body must lex as a Str token");
    assert_eq!(doc.slice(body.range()), r#""[a-z]+""#);
    assert!(!has_lexical_error(&doc));
}

#[test]
fn invalid_escape_bounds_the_unterminated_run() {
    // `\x` is not in the Str escape set, so the match dies at the
    // backslash: the residue up to and including it is one
    // UnterminatedStr token, and lexing resumes at `x`. What the grammar
    // does with the pieces afterwards is ticket-04 recovery territory.
    let doc = parse(r#"<a> ::= "bad \x escape";"#);
    let mut tokens = doc.tokens();
    let residue = tokens
        .find(|token| token.kind() == TokenKind::UnterminatedStr)
        .expect("the string residue must be retained");
    assert_eq!(doc.slice(residue.range()), r#""bad \"#);
    let after = tokens.next().expect("lexing continues past the residue");
    assert_eq!(after.kind(), TokenKind::Id);
    assert_eq!(doc.slice(after.range()), "x");
    assert!(doc
        .errors()
        .iter()
        .any(|error| error.kind() == SyntaxErrorKind::UnterminatedString));
}

#[test]
fn unterminated_string_error_comes_before_the_grammar_error() {
    // Lexical errors are recorded in source order; the grammar error is
    // appended after lexing, so the residue's error comes first.
    let source = r#"<a> ::= "open"#;
    let doc = parse(source);
    let kinds: Vec<_> = doc.errors().iter().map(|error| error.kind()).collect();
    assert_eq!(
        kinds,
        vec![
            SyntaxErrorKind::UnterminatedString,
            SyntaxErrorKind::UnexpectedEof,
        ]
    );
    // The residue covers quote through end of file.
    let last = doc.tokens().last().expect("the residue is a token");
    assert_eq!(last.kind(), TokenKind::UnterminatedStr);
    assert_eq!(doc.slice(last.range()), "\"open");
    assert_eq!(last.range().end, source.len());
}
