//! Shared public-seam fixtures and document invariants for the
//! `bnfgen-syntax` integration tests.
//!
//! Fixtures live in `tests/fixtures/*.bnfgen`, one source per file, loaded
//! with `include_str!`. Do not inline grammar in test files. Pin CRLF via
//! `tests/fixtures/.gitattributes` when a case needs it; files may omit a
//! trailing newline (see `.editorconfig`).
//!
//! Each name is a file stem. Ticket-02 cases pin lexical phenomena;
//! additional files cover empty input, a missing terminator, and raw
//! lexemes (`Int`, `re`, string escapes) whose grammar errors will churn
//! when ticket 03 maps those terminals — that churn is the snapshot's job.

/// Named public-seam fixtures: (name, source). Names are file stems.
pub const FIXTURES: &[(&str, &str)] = &[
    // Empty source: zero tokens, no errors, no rules. `token_at(0)` is
    // `None` because there is no final token to answer with.
    ("empty", include_str!("../fixtures/empty.bnfgen")),
    // The ticket-01 happy path: one complete untyped rule.
    (
        "valid_untyped_rule",
        include_str!("../fixtures/valid_untyped_rule.bnfgen"),
    ),
    // Complete tokens, missing `;`: a grammar error with no UnterminatedStr.
    (
        "missing_semicolon",
        include_str!("../fixtures/missing_semicolon.bnfgen"),
    ),
    // Comment at end of file: the last token is a Comment ending at EOF,
    // with no trailing newline to terminate it (the legacy skip pattern
    // required one and could not consume this shape).
    (
        "comment_at_eof",
        include_str!("../fixtures/comment_at_eof.bnfgen"),
    ),
    // A comment mid-file between two recognized rules.
    (
        "comment_between_rules",
        include_str!("../fixtures/comment_between_rules.bnfgen"),
    ),
    // Unterminated string ending the file: quote-to-EOF residue. Lexical
    // error is recorded first; the grammar error is appended after lexing.
    (
        "unterminated_string_at_eof",
        include_str!("../fixtures/unterminated_string_at_eof.bnfgen"),
    ),
    // Unterminated string after a complete rule. Until ticket 04 adds
    // grammar recovery, any grammar error drops recognition for the whole
    // document, so the earlier rule survives in the token buffer only —
    // not in `rules()`. The snapshot pins that honest state.
    (
        "unterminated_string_after_rule",
        include_str!("../fixtures/unterminated_string_after_rule.bnfgen"),
    ),
    // A run of bytes no rule can start with: one Invalid token per byte,
    // both surrounding rules recognized.
    (
        "invalid_run_between_rules",
        include_str!("../fixtures/invalid_run_between_rules.bnfgen"),
    ),
    // A single `/` is not a comment (`//` is) — it is Invalid input.
    (
        "single_slash_is_invalid",
        include_str!("../fixtures/single_slash_is_invalid.bnfgen"),
    ),
    // CRLF line endings: every bare `\r` is one byte of Invalid input
    // (drift-free with the legacy lexer, which rejected CRLF files);
    // both rules still recognize. The file is pinned to CRLF in
    // `tests/fixtures/.gitattributes`.
    (
        "carriage_return_between_rules",
        include_str!("../fixtures/carriage_return_between_rules.bnfgen"),
    ),
    // Multibyte content inside a string literal: byte ranges over
    // multi-byte codepoints must stay safe for source slicing.
    (
        "multibyte_string_rule",
        include_str!("../fixtures/multibyte_string_rule.bnfgen"),
    ),
    // A multibyte byte run no rule accepts: one Invalid token spanning
    // the full codepoint, placed inside the right-hand side so grammar
    // recognition survives and the snapshot stays ticket-04-stable.
    (
        "invalid_multibyte_ident",
        include_str!("../fixtures/invalid_multibyte_ident.bnfgen"),
    ),
    // Whitespace only: a single token tiling the whole source.
    (
        "whitespace_only",
        include_str!("../fixtures/whitespace_only.bnfgen"),
    ),
    // 2^64 digits: the legacy lexer parsed at lex time and rejected this
    // with `InvalidInteger`. Here they are a raw Int lexeme; whether the
    // value fits is a downstream concern. Ticket 03 will change the
    // grammar error once `Int` is a mapped terminal.
    (
        "integer_overflow",
        include_str!("../fixtures/integer_overflow.bnfgen"),
    ),
    // Quotes retained, escapes as typed: nothing was decoded.
    (
        "string_escapes",
        include_str!("../fixtures/string_escapes.bnfgen"),
    ),
    // `re` plus a raw string body. The syntax crate neither compiles nor
    // validates the regular expression; ticket 03 will map `Re`.
    ("re_literal", include_str!("../fixtures/re_literal.bnfgen")),
    // `\x` is not in the Str escape set, so the match dies at the
    // backslash: UnterminatedStr residue, then lexing resumes at `x`.
    (
        "invalid_escape",
        include_str!("../fixtures/invalid_escape.bnfgen"),
    ),
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

/// Assert `token_at` totality over a document: every offset in
/// `0..=source.len()` answers (except the empty document, which answers
/// `None` at 0), past-end is `None`, and interior answers contain the
/// offset. Boundary offsets resolve to the token that starts there because
/// containment is `start <= offset < end`.
pub fn assert_token_lookup(doc: &bnfgen_syntax::ParsedDocument) {
    let len = doc.source().len();
    assert!(
        doc.token_at(len + 1).is_none(),
        "token_at past the end of the source must be None",
    );
    if len == 0 {
        assert!(
            doc.token_at(0).is_none(),
            "an empty document has no token at offset 0",
        );
        return;
    }
    for offset in 0..len {
        let token = doc
            .token_at(offset)
            .unwrap_or_else(|| panic!("token_at({offset}) must answer inside a non-empty source"));
        let range = token.range();
        assert!(
            range.start <= offset && offset < range.end,
            "token_at({offset}) returned {range:?}, which does not contain \
             the offset (containment is start <= offset < end; a boundary \
             belongs to the token that starts there)",
        );
        doc.slice(range);
    }
    let last = doc
        .token_at(len)
        .expect("end of source answers with the final token");
    assert_eq!(
        last.range().end,
        len,
        "the token at end of source must end at the source length",
    );
    doc.slice(last.range());
}
