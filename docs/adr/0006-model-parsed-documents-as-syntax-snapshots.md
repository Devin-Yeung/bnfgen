# Model parsed documents as syntax snapshots

**Status: accepted**

`ParsedDocument` remains the root result of syntax parsing, but it is defined as a lossless snapshot of one source text rather than as a partial semantic grammar. It owns source text and the complete token buffer, exposes source-established syntax facts through language-specific views, and retains syntax diagnostics and recovery observations separately. It contains neither resolved symbol identities nor a rule-dependency graph; `bnfgen-analysis` derives those semantic facts from its views, and `bnfgen` derives a start-specific `GenerationPlan` from analysis.

## Consequences

A syntax view may be partial and must say only what source establishes: a missing delimiter or child remains absent rather than becoming a fabricated token, semantic value, or graph edge. Parser recovery must distinguish a diagnostic about missing or unexpected syntax from source text it consumed or could not attach structurally; a future public recovery view replaces bare recovery ranges when recovery kind or ownership becomes relevant. The parser records structural completeness and recovery status privately per recognized fact so analysis can choose trustworthy facts without reconstructing parser state from raw tokens or overlapping ranges.

The public surface remains small: source access, source-ordered tokens, typed syntax views, syntax diagnostics, recovery observations, token lookup, and cursor-context classification. Generic concrete-tree traversal, decoded literals, symbol resolution, graph nodes, graph edges, and generation readiness do not enter `ParsedDocument`. Existing parse-and-query snapshots remain the compatibility contract, but their normalized rendering must show diagnostics, recovery observations, and partial syntax as distinct categories rather than treating a recovery range as an implicit error tree.
