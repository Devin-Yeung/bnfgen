# 08 — Adopt `bnfgen-syntax` in the strict parsing pipeline

**What to build:** Redirect the existing generation-facing parse entry point through `bnfgen-syntax`. Complete syntax is strictly lowered outside the syntax crate into the current grammar model, while existing library, CLI, diagnostic, and seeded generation behavior remains stable.

**Blocked by:** 04 — Recover broken rules without losing later rules.

**Status:** ready-for-agent

- [ ] The active strict parse path obtains source structure from `bnfgen-syntax` rather than invoking the legacy lexer and parser directly.
- [ ] A compatibility lowering outside `bnfgen-syntax` decodes literals, parses integers, compiles regular expressions, and constructs the current complete grammar model.
- [ ] Incomplete or invalid syntax is rejected by the strict entry point using current caller-visible error behavior while the syntax parse itself remains total.
- [ ] Existing valid parser snapshots, rendered diagnostic snapshots, CLI behavior, and seeded generation outputs remain unchanged unless a separately approved parser bug is identified.
- [ ] The compatibility lowering is documented as temporary migration debt with a TODO for the later analysis redesign.
- [ ] No generation, Miette, Petgraph, Rand, CLI, or LSP dependency is introduced into `bnfgen-syntax`.
- [ ] The legacy parser may remain present for the contraction ticket, but it is no longer an active source of truth for strict parsing.
- [ ] Workspace and documentation tests pass through the redirected pipeline.
