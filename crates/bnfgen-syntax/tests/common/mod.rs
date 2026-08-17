//! Shared public-seam fixtures and lexical invariants for the
//! `bnfgen-syntax` integration tests.
//!
//! Fixtures are (name, source) pairs exercised through `parse` only. Each
//! names a lexical phenomenon from ticket 02; the snapshot test pins their
//! exact token shapes. Sources deliberately avoid significant kinds the
//! ticket-01 grammar does not map yet (`Int`, `re`, `{`, `(`, `:`) — their
//! grammar errors would churn the moment ticket 03 wires those terminals
//! in, and their raw-lexeme proof lives in `raw_lexemes.rs` instead.

/// Named public-seam fixtures: (name, source).
pub const FIXTURES: &[(&str, &str)] = &[
    // The ticket-01 happy path: one complete untyped rule.
    (
        "valid_untyped_rule",
        "<greeting> ::= \"hello\" <name> | \"bye\";\n",
    ),
    // Comment at end of file: the last token is a Comment ending at EOF,
    // with no trailing newline to terminate it (the legacy skip pattern
    // required one and could not consume this shape).
    ("comment_at_eof", "<a> ::= \"x\"; // trailing note"),
    // A comment mid-file between two recognized rules.
    (
        "comment_between_rules",
        "<a> ::= \"x\"; // mid-file\n<b> ::= \"y\";\n",
    ),
    // Unterminated string ending the file: quote-to-EOF residue.
    ("unterminated_string_at_eof", "<a> ::= \"unterminated"),
    // Unterminated string after a complete rule. Until ticket 04 adds
    // grammar recovery, any grammar error drops recognition for the whole
    // document, so the earlier rule survives in the token buffer only —
    // not in `rules()`. The snapshot pins that honest state.
    (
        "unterminated_string_after_rule",
        "<a> ::= \"x\";\n<b> ::= \"open",
    ),
    // A run of bytes no rule can start with: one Invalid token per byte,
    // both surrounding rules recognized.
    (
        "invalid_run_between_rules",
        "<a> ::= \"x\"; @#~ <b> ::= \"y\";\n",
    ),
    // A single `/` is not a comment (`//` is) — it is Invalid input.
    (
        "single_slash_is_invalid",
        "<a> ::= \"x\"; / <b> ::= \"y\";\n",
    ),
    // CRLF line endings: every bare `\r` is one byte of Invalid input
    // (drift-free with the legacy lexer, which rejected CRLF files);
    // both rules still recognize.
    (
        "carriage_return_between_rules",
        "<a> ::= \"x\";\r\n<b> ::= \"y\";\r\n",
    ),
    // Multibyte content inside a string literal: byte ranges over
    // multi-byte codepoints must stay safe for source slicing.
    ("multibyte_string_rule", "<a> ::= \"héllo wörld\";\n"),
    // A multibyte byte run no rule accepts: one Invalid token spanning
    // the full codepoint, placed inside the right-hand side so grammar
    // recognition survives and the snapshot stays ticket-04-stable.
    ("invalid_multibyte_ident", "<a> ::= \"x\" ü;\n"),
    // Whitespace only: a single token tiling the whole source.
    ("whitespace_only", " \t\n"),
];

/// Assert the ticket-02 invariant in executable form: the token buffer
/// tiles the source completely — ordered, contiguous, in bounds, and safe
/// for slicing. This is "every source character is observable" as a
/// property, independent of any fixture's specific shapes.
pub fn assert_tiles_source(doc: &bnfgen_syntax::ParsedDocument) {
    let mut previous_end = 0;
    for (index, token) in doc.tokens().enumerate() {
        let range = token.range();
        assert_eq!(
            range.start, previous_end,
            "token #{index} starts at {} but the previous token ended at \
             {previous_end}; the buffer must tile the source without gaps \
             or overlap",
            range.start,
        );
        // `slice` asserts bounds internally and would panic on a
        // non-char-boundary slice, so a successful call proves the range
        // is safe to hand to callers.
        doc.slice(range.clone());
        previous_end = range.end;
    }
    assert_eq!(
        previous_end,
        doc.source().len(),
        "the final token ends at {previous_end} but the source is {} bytes",
        doc.source().len(),
    );
}
