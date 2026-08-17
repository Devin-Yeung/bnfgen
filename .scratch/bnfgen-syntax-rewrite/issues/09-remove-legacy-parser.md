# 09 — Remove the legacy parser and close the migration

**What to build:** Complete the contraction phase by deleting the generation crate's duplicate source-language implementation. `bnfgen-syntax` becomes the sole owner of Logos, LALRPOP, tokenization, parsing, and recovery while all public behavior and dependency rules remain verified.

**Blocked by:** 07 — Harden total parsing across real typing states; 08 — Adopt `bnfgen-syntax` in the strict parsing pipeline.

**Status:** ready-for-agent

- [ ] The generation-facing crate no longer contains a duplicate Logos lexer, LALRPOP grammar, parser glue, parse-error conversion, or private lexer snapshots.
- [ ] `bnfgen-syntax` is the only workspace crate that owns source-language tokenization and parsing dependencies.
- [ ] No caller continues to invoke or reference the legacy parser path.
- [ ] Public syntax behavior remains covered solely through the parse-and-query seam rather than migrated private tests.
- [ ] Existing grammar, generator, CLI, MCP, documentation, and seeded-output tests all pass after removal.
- [ ] Dependency checks confirm that `bnfgen-syntax` remains independent of Rowan, Miette, Petgraph, Rand, generation, CLI, and LSP concerns.
- [ ] The compatibility-lowering TODO remains explicit for the later `bnfgen-analysis` redesign and no other migration-only duplication remains.
- [ ] Architecture documentation and the parent spec accurately describe the final ownership and migration state.
