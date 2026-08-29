# Use one handwritten tolerant syntax parser

**Status: accepted**

`bnfgen-syntax` will replace its strict LALRPOP grammar plus fallback partial-rule LALRPOP grammar with one private handwritten recursive-descent parser over the complete Logos token buffer's significant-token view. The parser produces the existing token-backed `ParsedDocument`: recognized range-backed syntax records, structured syntax errors, and recovery ranges. This changes parser implementation, not the syntax crate's external seam: parsing remains total; source, every raw token, and only source-established partial facts remain queryable through immutable language-specific views.

## Context

Bnfgen must parse documents while they are being edited. A rule can be missing a delimiter, contain unexpected input, or end mid-symbol while subsequent rules remain valid and useful. The current implementation first uses a strict LALRPOP grammar to find a failed range, then applies a second LALRPOP prefix grammar to recover partial facts from that range. The two grammars duplicate non-terminals, alternatives, symbols, repeats, and regular expressions; every language change must keep complete syntax and each typing-state variation consistent. Recovery ownership is divided between generated-parser recovery, range selection in the document layer, and the fallback grammar.

Mature editor parsers vary in algorithm, but their durable shape is one authoritative parser that handles normal recognition, missing syntax, unexpected text, and local synchronization together. Rust-analyzer is the relevant fit for Bnfgen: its handwritten recursive-descent parser records errors, consumes unexpected input into error regions when necessary for progress, and preserves caller-owned synchronization tokens. Tree-sitter and Lezer achieve the same single-result property with richer generated parser runtimes, which LALRPOP does not supply here. See [`../research/tolerant-parsing-practice.md`](../research/tolerant-parsing-practice.md).

## Decision

The private parser will have construct-level functions such as `parse_rule`, `parse_non_terminal`, `parse_alternative`, `parse_symbol`, and `parse_repeat`. Each function records only syntax observed in source. It reports a missing expected element without fabricating a token or complete generation object; it places unexpected consumed tokens in a recovery region; and it uses construct-specific synchronization sets so a nested parser leaves `>`, `}`, `)`, `|`, `;`, and a credible following rule start to their owners when appropriate. Every recovery loop must either consume input, return, or make a missing-syntax decision, preventing non-progress loops.

`ParsedDocument` is the final output of the **syntax stage**, not the final semantic grammar or generation input. `bnfgen-analysis` consumes its typed views, decodes and resolves complete facts, reports semantic diagnostics, and constructs the rule-dependency graph. The generation crate creates a start-specific `GenerationPlan` only from analysis that establishes generation invariants. Incomplete syntax can still support editor queries and best-effort analysis, but it cannot become a generation-ready model merely because a parser recovered around it.

## Consequences

The Logos lexer, source-backed token buffer, range-backed private records, immutable views, and public parse-and-query test suite remain. Existing normalized fixtures are the migration contract: valid syntax must retain its visible structure; incomplete syntax must retain truthful partial facts; malformed input must retain errors, recovery ranges, and later valid rules. Parser implementation tests may change or disappear, but public snapshots, prefix tests, mutation tests, range invariants, and downstream generation regressions remain.

Full-document reparsing remains the initial performance model. This decision does not adopt Rowan, Tree-sitter, a generic concrete syntax tree, incremental parsing, or a generic parser framework. It replaces only the two coordinated LALRPOP grammars inside `bnfgen-syntax`; the later compatibility lowering and removal of the legacy parser in `bnfgen` remain required migration work.

This supersedes the LALRPOP-specific implementation clauses of ADR-0002 and ADR-0003. Their crate seams, token-backed document model, total parsing contract, and separation between syntax, analysis, and generation remain in force.
