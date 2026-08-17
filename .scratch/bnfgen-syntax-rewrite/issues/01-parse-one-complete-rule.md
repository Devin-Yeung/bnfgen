# 01 — Parse one complete rule through `bnfgen-syntax`

**What to build:** Establish the independent syntax crate and its public parse-and-query seam. A caller can parse empty input, malformed input, and a simple complete rule into a source-backed `ParsedDocument`, then observe source, tokens, typed rule structure, and syntax errors without receiving a failed parse result.

**Blocked by:** None — can start immediately.

**Status:** ready-for-agent

- [ ] The workspace contains an independent `bnfgen-syntax` crate that builds without depending on analysis, generation, CLI, LSP, Rowan, Miette, Petgraph, or Rand concerns.
- [ ] The public parse operation returns a `ParsedDocument` for empty input, a simple complete rule, and malformed input without panicking or returning a failed parse result.
- [ ] A caller can recover the retained source, iterate the simple rule's source-ordered tokens, and query a language-specific typed rule view through public interfaces.
- [ ] Syntax errors are retained by the document and are empty for the supported complete-rule fixture.
- [ ] Storage identifiers and parser-generator types remain private.
- [ ] A compile-time assertion proves that `ParsedDocument` is `Send + Sync`.
- [ ] Tests exercise only the public parse-and-query seam and the workspace remains green.
