//! `token_at` semantics (ticket 02): containment, boundaries, end of
//! source, and totality — always an answer inside the source, never a
//! panic, never an internal identifier.

use bnfgen_syntax::{parse, TokenKind};

#[test]
fn offset_zero_returns_the_first_token() {
    let doc = parse("<a> ::= \"x\";");
    let token = doc.token_at(0).expect("offset 0 is inside the first token");
    assert_eq!(token.kind(), TokenKind::LAngle);
    assert_eq!(token.range(), 0..1);
}

#[test]
fn a_boundary_offset_resolves_to_the_token_starting_there() {
    // `<a>`: the Id `a` occupies 1..2 and `>` starts at 2. Offset 2 is
    // the shared boundary; the token that starts there wins.
    let doc = parse("<a> ::= \"x\";");
    let token = doc.token_at(2).expect("offset 2 is a token boundary");
    assert_eq!(token.kind(), TokenKind::RAngle);
    assert_eq!(token.range(), 2..3);
}

#[test]
fn a_mid_token_offset_returns_the_containing_token() {
    // `<ab>`: the Id `ab` occupies 1..3; offset 2 is strictly inside it.
    let doc = parse("<ab> ::= \"x\";");
    let token = doc.token_at(2).expect("offset 2 is inside an Id token");
    assert_eq!(token.kind(), TokenKind::Id);
    assert_eq!(token.range(), 1..3);
    assert_eq!(doc.slice(token.range()), "ab");
}

#[test]
fn a_multibyte_interior_offset_returns_the_containing_token() {
    // `"héllo"`: the `é` occupies bytes 10..12; offset 11 is strictly
    // inside the codepoint, still inside the Str token.
    let doc = parse("<a> ::= \"héllo\";");
    let token = doc.token_at(11).expect("offset 11 is inside the string");
    assert_eq!(token.kind(), TokenKind::Str);
    assert_eq!(doc.slice(token.range()), "\"héllo\"");
}

#[test]
fn end_of_source_resolves_to_the_final_token() {
    let eof_comment = parse("<a> ::= \"x\"; // note");
    let last = eof_comment
        .token_at(eof_comment.source().len())
        .expect("end of source answers with the final token");
    assert_eq!(last.kind(), TokenKind::Comment);
    assert_eq!(last.range().end, eof_comment.source().len());

    let blank = parse(" \t\n");
    let token = blank
        .token_at(3)
        .expect("end of a whitespace-only source answers too");
    assert_eq!(token.kind(), TokenKind::Whitespace);
    assert_eq!(token.range(), 0..3);
}

#[test]
fn past_end_and_empty_documents_return_none() {
    let doc = parse("<a> ::= \"x\";");
    assert!(doc.token_at(doc.source().len() + 1).is_none());

    let empty = parse("");
    assert!(empty.token_at(0).is_none());
}

#[test]
fn lookup_is_total_and_sliceable_across_the_whole_source() {
    // Every offset from 0 through end of file answers with a token whose
    // range slices cleanly — the lookup totality that ticket 07 fuzzes at
    // scale, pinned here on one fixture holding every token class:
    // significant, multibyte string, invalid input, and comment.
    let source = "<a> ::= \"héllo\"; @ // c";
    let doc = parse(source);
    for offset in 0..=source.len() {
        let token = doc
            .token_at(offset)
            .unwrap_or_else(|| panic!("token_at({offset}) must answer inside the source"));
        doc.slice(token.range());
    }
}
