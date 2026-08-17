# Separate document analysis from generation planning

Bnfgen will replace the public `RawGrammar -> CheckedGrammar` pipeline with an always-available document analysis and a start-specific `GenerationPlan`. The plan is the proof that the subgraph reachable from one Start Symbol satisfies the invariants required by generation; syntax recovery, diagnostics, and graph implementation remain outside that execution model. This is a breaking redesign because the existing `CheckedGrammar` does not include start-dependent analyses and therefore cannot honestly mean "ready for generation".

## Consequences

The intended public interface is limited to document, analysis, structured diagnostics, generation plans, and generators. Existing output-oriented and seeded-generation snapshots should be reused; parser recovery gains typing-state snapshots, while private data layouts are not treated as stable snapshot contracts.
