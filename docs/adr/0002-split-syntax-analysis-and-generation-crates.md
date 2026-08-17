# Split syntax, analysis, and generation crates

Bnfgen will split its front end into `bnfgen-syntax` and `bnfgen-analysis`, while the existing `bnfgen` package becomes the generation-facing library and `bnfgen-cli` remains an adapter. This split is part of the parser refactor, not deferred LSP work: it lets a future `bnfgen-lsp` depend directly on syntax and analysis without importing generation or CLI concerns, while keeping the LALRPOP parser behind the syntax interface.

## Consequences

Graph algorithms and structured diagnostics belong to analysis; parsing, recovery, tokens, source ranges, and language-specific syntax views belong to syntax; generation plans and generators belong to `bnfgen`; rendering and transport belong to adapters. `bnfgen-analysis` consumes the syntax views instead of reconstructing grammar structure from raw token kinds, so parser knowledge remains local to the independent `bnfgen-syntax` crate. We deliberately reject separate parser, graph, and diagnostics crates because those would expose implementation details through shallow interfaces. The LSP crate itself is deferred until the parser and analysis foundations are complete.
