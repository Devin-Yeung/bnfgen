# 05 — Retain typed facts inside incomplete rules

**What to build:** Preserve useful syntax inside the rule currently being edited. Callers can query the left-hand side, alternatives, or partial non-terminal reference already present in a malformed rule, while missing required syntax is reported through optional typed accessors.

**Blocked by:** 04 — Recover broken rules without losing later rules.

**Status:** ready-for-agent

- [x] Incomplete rule declarations retain any recognized left-hand-side name and type syntax.
- [x] Incomplete right-hand sides retain recognized alternatives, symbols, and non-terminal reference prefixes where the source establishes them.
- [x] Missing closing delimiters, definition syntax, symbols, repeat bounds, or rule terminators appear as absent typed children rather than fabricated values.
- [x] Representative typing states include incomplete typed non-terminals, non-terminal references, literals, regular expressions, alternatives, and repeat ranges.
- [x] Analysis-style callers can obtain recognized facts entirely through typed views and never need to reconstruct grammar structure from raw token patterns.
- [x] Syntax errors and recovery ranges remain separately queryable from the recognized partial facts.
- [x] Normalized fixtures demonstrate that adding the missing syntax turns the partial views into complete views without changing unrelated ranges.
- [x] No semantic decoding, resolution, graph analysis, or generation validation enters the syntax crate.

## Comments

2026-08-30 — Implemented on branch `bnfgen-syntax`. `ParsedDocument::rules()` is the single tolerant interface for complete and incomplete rules. Required syntax is represented by optional typed children, while errors and recovery ranges remain independent queries. A private LALRPOP prefix grammar recognizes facts only inside ranges isolated by the strict grammar, avoiding token-pattern reconstruction in callers or the document layer.

Representative public snapshots cover incomplete declarations, right-hand sides, typed references, unterminated terminals, regular expressions, alternatives, repeat bounds, a complete rule before an incomplete rule, and an incomplete rule before a later valid rule.

Verification: `cargo test --workspace`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo fmt --all --check`, and `git diff --check` pass.
