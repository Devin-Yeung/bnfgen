# 01 — Parse one complete rule through `bnfgen-syntax`

**What to build:** Establish the independent syntax crate and its public parse-and-query seam. A caller can parse empty input, malformed input, and a simple complete rule into a source-backed `ParsedDocument`, then observe source, tokens, typed rule structure, and syntax errors without receiving a failed parse result.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [x] The workspace contains an independent `bnfgen-syntax` crate that builds without depending on analysis, generation, CLI, LSP, Rowan, Miette, Petgraph, or Rand concerns.
- [x] The public parse operation returns a `ParsedDocument` for empty input, a simple complete rule, and malformed input without panicking or returning a failed parse result.
- [x] A caller can recover the retained source, iterate the simple rule's source-ordered tokens, and query a language-specific typed rule view through public interfaces.
- [x] Syntax errors are retained by the document and are empty for the supported complete-rule fixture.
- [x] Storage identifiers and parser-generator types remain private.
- [x] A compile-time assertion proves that `ParsedDocument` is `Send + Sync`.
- [x] Tests exercise only the public parse-and-query seam and the workspace remains green.

## Comments

2026-08-17 — Implemented on branch `bnfgen-syntax` (crate `crates/bnfgen-syntax`, deps: `logos` + `lalrpop-util` only). Notes for the reviewer:

- The token model is raw from the start: no callbacks decode integers or strings, so ticket 02's raw-lexeme requirement is already the storage shape, not a retrofit. Whitespace and comments are real token kinds (no `skip`), keeping the buffer source-preserving per ADR 0003.
- The LALRPOP grammar recognizes only the untyped rule form (`<id> ::= alternatives ;`) — the deliberate ticket-01 slice. Typed heads, weights, repeat ranges, and `re(...)` symbols arrive with ticket 03. A grammar error currently drops all recognized rules (`Vec::new()` with a TODO); ticket 04 replaces that with recovery.
- The legacy `Id` regex `[a-zA-Z-_0-9]+` was copied verbatim, quirks included (`-_` parses as the range `A-_`), so lexing cannot drift during migration.
- On any parse failure the parser's `expected` terminal list is dropped rather than retained as message strings; the TODO marks where ticket 04 should surface structured expected-terminal data.
- Lexically invalid input is recorded as a retained `UnrecognizedInput` error and skipped from the buffer; ticket 02 gives it token representation.
- Invalid-chars-at-EOF note for ticket 02: the comment kind is `//[^\n]*` (newline-exclusive, EOF-safe), unlike the legacy skip pattern which required a trailing newline.

Verification: `cargo test --workspace` green (6 new public-seam tests in `crates/bnfgen-syntax/tests/parse_and_query.rs`), `cargo clippy --workspace --all-targets -- -D warnings` clean, `cargo fmt --all --check` clean. `bnfgen-core` and `bnfgen-cli` untouched.
