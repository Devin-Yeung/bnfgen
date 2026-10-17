# Front-end crate boundaries

Bnfgen's front end is divided by responsibility across three library crates.

- `bnfgen-syntax` owns the source language: lexing, grammar recognition, recovery, and the `ParsedDocument` that callers query through immutable views. It knows the shape of a grammar file, not its meaning.
- `bnfgen-analysis` consumes those views to decode values, resolve names, report semantic diagnostics, and build the rule-dependency graph.
- `bnfgen` (generation) creates a start-specific `GenerationPlan` only from analysis that establishes the generation invariants for that start symbol's reachable subgraph. There is no always-ready "checked grammar" object.

Adapters such as `bnfgen-cli` and a future language server depend on these crates. The crates do not depend on the adapters.

## Consequences

- `bnfgen-syntax` does not depend on analysis, generation, CLI, or protocol code, nor on Rowan, Miette, Petgraph, or Rand.
- `bnfgen-analysis` consumes typed syntax views only. It never matches raw token kinds or reads private parser records to reconstruct grammar structure.
- Parsing, recovery, and graph analysis are not split into separate parser, graph, or diagnostics crates, because those would expose implementation details through shallow interfaces. The lexer and parser are private modules of `bnfgen-syntax`.
- Generation planning is per start symbol. A plan proves that one start symbol's reachable subgraph satisfies the generation invariants; syntax recovery and diagnostics stay outside that execution model.
