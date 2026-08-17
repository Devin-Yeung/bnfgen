# Rewrite `bnfgen-syntax`

Status: ready-for-agent

## Problem Statement

Bnfgen's current parser is inseparable from the generation model. It skips trivia, decodes literals during lexing, compiles regular expressions during parsing, and fails the whole document on the first lexical, syntactic, or fallible parser-action error. That behavior is adequate for a batch generator, but it cannot support editor navigation and completion while a grammar is temporarily incomplete.

The project needs an independent `bnfgen-syntax` crate that owns the source language and tolerant parsing without importing semantic analysis, generation, CLI rendering, or LSP protocol concerns. The crate must preserve every source token and expose useful recognized structure for both valid and incomplete documents. It must do this through language-specific immutable views, not through Rowan or another generic concrete syntax tree.

## Solution

Build `bnfgen-syntax` as the sole owner of the Logos lexer and LALRPOP grammar. Its total parse entry point returns a source-backed `ParsedDocument` for every input. The document retains the original source, a complete token buffer, syntax errors, recovery information, and private token-or-range-backed records for recognized grammar structure.

Callers observe the document through one public seam: parsing followed by immutable queries for tokens, rules and their typed children, syntax errors, token lookup, and syntax context at a byte offset. Required syntax that is missing is represented by an absent typed child rather than by fabricating a valid generation object. Literal decoding, regular-expression compilation, name resolution, semantic diagnostics, graph analysis, and generation invariants remain downstream.

During migration, the existing generation-facing library continues to preserve its current public behavior through a temporary strict-lowering adapter outside `bnfgen-syntax`. By the end of this work, the old lexer and parser implementation no longer remain duplicated in the generation crate.

## User Stories

1. As an editor user, I want an incomplete grammar to remain parseable as a document, so that navigation does not disappear while I type.
2. As an editor user, I want a malformed rule not to discard later valid rules, so that an error remains local to the text I am editing.
3. As an editor user, I want completion to understand that I am typing a non-terminal reference even before its closing delimiter exists, so that useful suggestions appear during normal typing.
4. As an editor user, I want comments and whitespace to remain associated with their exact source locations, so that cursor queries reflect the document I can see.
5. As an editor user, I want invalid and unterminated input to remain represented in the parsed document, so that the server does not silently lose text.
6. As an editor user, I want syntax context to distinguish a rule name, type annotation, right-hand-side symbol, repeat range, literal, and space between rules, so that completion can be relevant to the cursor position.
7. As an editor user, I want edits containing multibyte UTF-8 text to produce valid byte ranges, so that navigation never points outside the source.
8. As an LSP author, I want a total parse operation, so that document state does not need a separate parsing-failed variant.
9. As an LSP author, I want token lookup at a byte offset, so that hover, completion, and navigation can begin from the current source position.
10. As an LSP author, I want syntax-context classification at a byte offset, so that protocol code does not need to interpret raw token sequences.
11. As an LSP author, I want the parsed document to be `Send + Sync`, so that document snapshots can be retained and queried across worker threads.
12. As an LSP author, I want line, column, and UTF-16 conversion to remain outside syntax tokens, so that one protocol-specific coordinate system is not embedded in the language model.
13. As an analysis author, I want to iterate recognized rules through typed syntax views, so that analysis does not depend on parser storage.
14. As an analysis author, I want typed access to a rule's left-hand side, alternatives, and symbols, so that semantic indexing does not reconstruct grammar structure from punctuation.
15. As an analysis author, I want missing required children to be returned as absent values, so that best-effort analysis can skip only the facts it cannot establish.
16. As an analysis author, I want typed and untyped non-terminal syntax to preserve its raw spelling and range, so that symbol identity can be resolved downstream.
17. As an analysis author, I want terminal, integer, type, and regular-expression lexemes to remain undecoded, so that decoding and validation produce semantic diagnostics at the correct downstream seam.
18. As an analysis author, I want syntax errors and recovered regions to be queryable independently of recognized rules, so that semantic work can proceed on unaffected syntax.
19. As a library caller, I want valid grammars to retain their existing generated behavior, so that the parser refactor does not change seeded outputs.
20. As a CLI user, I want existing checks and generation commands to keep working while the syntax crate is introduced, so that the migration can be delivered incrementally.
21. As a CLI user, I want existing rendered diagnostics to remain behaviorally stable during this syntax-only rewrite, so that diagnostic redesign can happen separately.
22. As a syntax crate caller, I want tokens to be returned in source order, so that iteration is predictable and source slices can be reconstructed without sorting.
23. As a syntax crate caller, I want each token to expose its kind and UTF-8 byte range, so that its raw text has one canonical source of truth.
24. As a syntax crate caller, I want the original source retained by the parsed document, so that token text does not need to be copied into every token.
25. As a syntax crate caller, I want syntax errors to carry stable source ranges and structured kinds, so that callers can reason about errors without parsing messages.
26. As a syntax crate caller, I want rule and symbol views to borrow from the document, so that callers cannot retain stale syntax records independently of their source.
27. As a syntax crate caller, I want private storage changes to leave the public views unchanged, so that parser recovery can improve without forcing downstream rewrites.
28. As a maintainer, I want Logos and LALRPOP to live in one crate, so that source-language knowledge has one owner.
29. As a maintainer, I want LALRPOP to consume a significant-token view over the complete token buffer, so that trivia retention and grammar recognition remain separate internal concerns.
30. As a maintainer, I want lexical failures to become retained tokens or recoverable syntax facts rather than iterator failures, so that one invalid character does not terminate the document.
31. As a maintainer, I want recovered parser input to remain represented by token identities or ranges, so that recovery never drops source text.
32. As a maintainer, I want later valid rules to survive recovery from a preceding broken rule, so that synchronization behavior has an observable correctness contract.
33. As a maintainer, I want every prefix of representative grammar examples to parse without panicking, so that ordinary typing states receive systematic coverage.
34. As a maintainer, I want mutations around delimiters, strings, comments, alternatives, and repeat ranges to preserve valid ranges, so that recovery is robust beyond hand-picked examples.
35. As a maintainer, I want normalized syntax snapshots rather than private `Debug` snapshots, so that internal record changes do not create meaningless test churn.
36. As a maintainer, I want no direct dependency on Rowan, Miette, Petgraph, Rand, generation code, CLI code, or LSP types, so that `bnfgen-syntax` remains an independent language module.
37. As a maintainer, I want full-document reparsing to be the initial performance model, so that incremental storage machinery is not introduced without evidence.
38. As a maintainer, I want the temporary compatibility lowering to be visibly temporary and outside `bnfgen-syntax`, so that generation concerns cannot migrate back into the syntax crate.
39. As a test author, I want all new syntax behavior testable through the public parse-and-query seam, so that tests describe caller-visible behavior rather than parser machinery.
40. As a future formatter author, I want source preservation not to depend on a generic tree, so that a later formatting design can choose its own justified representation.

## Implementation Decisions

- The accepted crate topology and responsibilities are governed by the existing front-end architecture and ADRs. This spec implements only the independent syntax portion and the compatibility work necessary to adopt it.
- `bnfgen-syntax` becomes the sole owner of Logos tokenization, the LALRPOP grammar, parser recovery, source ranges, syntax errors, `ParsedDocument`, and language-specific syntax views.
- Parsing is total. Every source string produces a `ParsedDocument`; syntax errors are data retained by that document rather than a failed parse result.
- `ParsedDocument` retains the original source and a complete source-ordered token buffer. The buffer includes significant tokens, whitespace, comments, invalid input, and recoverable unterminated forms.
- Token text is always obtained by slicing the retained source with a UTF-8 byte range. Tokens do not own decoded or duplicated text.
- The complete token buffer is the source-preserving representation. Recognized syntax and recovery are stored privately as references to token identities or byte ranges.
- The crate does not materialize a generic concrete syntax tree. Rowan, green/red nodes, tree sinks, syntax fragments, and trivia reinsertion are not part of the design.
- The public interface exposes language-specific immutable views for documents, rules, alternatives, symbols, and non-terminals. Views borrow from the parsed document and hide storage identifiers.
- Typed accessors return absent values when incomplete syntax lacks a required child. The syntax crate never fabricates a complete generation object from incomplete input.
- The public document queries cover source access, source-ordered token iteration, rule iteration, syntax errors, token lookup at a byte offset, and syntax-context classification at a byte offset.
- Cursor context is language-specific. It distinguishes at least rule declarations, left-hand-side names and types, definition syntax, right-hand-side symbols, non-terminal references and their types, literal or regular-expression input, repeat ranges, inter-rule space, and unknown or recovered regions.
- LALRPOP consumes an internal significant-token adapter over the complete token buffer. Trivia remains in the buffer and is never reinserted after parsing.
- Lexer failures that currently terminate the LALRPOP token stream become retained token kinds or equivalent recoverable syntax input. Unterminated strings and comments at end of file receive explicit typing-state coverage.
- Recovery must preserve all source tokens, retain syntax errors, and resume in time to recognize later valid rules. The exact recovery productions and synchronization algorithm remain private implementation details.
- Syntax parsing preserves raw integer, string, type, and regular-expression lexemes. It does not parse integers, decode escapes, compile regular expressions, resolve names, or establish generation invariants.
- `ParsedDocument` is `Send + Sync`; this property is protected with a compile-time assertion through the public crate.
- Token and syntax ranges use UTF-8 byte offsets. Line, column, UTF-16, and protocol position indexing remain a separate adapter concern.
- `bnfgen-analysis` is expected to consume typed syntax views, not raw token patterns or private recovery records. That downstream crate is not implemented by this spec.
- Full-document lexing and parsing on each edit is the initial performance model. No incremental reparsing or persistent subtree reuse is introduced.
- Migration proceeds as a vertical slice: establish the independent crate and public seam, port source retention and lexing, port tolerant LALRPOP parsing and views, then redirect the existing strict parsing path through a compatibility lowering outside `bnfgen-syntax`.
- By completion, the generation-facing crate no longer contains a duplicate Logos lexer or LALRPOP grammar. `bnfgen-syntax` is their single owner.
- Existing `RawGrammar`, `CheckedGrammar`, generator, CLI, and diagnostic interfaces remain available during this spec. A temporary strict-lowering adapter may construct those current models only from sufficiently complete syntax views.
- The compatibility lowering is deliberate migration debt. It must be documented with a TODO pointing to the later analysis redesign and must not introduce generation dependencies into `bnfgen-syntax`.
- Existing valid-grammar behavior, seeded generation output, and CLI behavior remain unchanged unless a current behavior is proven to depend on a parser bug and separately approved.
- Public storage structs, LALRPOP recovery types, internal token identifiers, and significant-token adapters do not cross the crate interface.

## Testing Decisions

- The single new testing seam is the public parse operation followed by `ParsedDocument` queries. Tests exercise external behavior through this seam and do not assert on Logos callbacks, LALRPOP productions, private token identifiers, recovery records, or storage layout.
- Valid-document tests assert source-ordered complete tokens, normalized recognized syntax, empty syntax errors, typed view behavior, and source slicing.
- Incomplete-document tests assert that parsing still returns a document, errors are structured and ranged, unaffected syntax remains queryable, and missing children are absent.
- Recovery tests place malformed input before valid rules and assert that the later rules remain recognized and that every original token remains represented.
- Cursor-context fixtures cover every meaningful position within rule declarations, typed and untyped non-terminals, alternatives, terminals, regular expressions, repeat ranges, comments, whitespace, malformed constructs, and end of file.
- Typing-state tests parse every prefix of representative grammar examples. No prefix may panic, and all token and error ranges must remain within the retained source.
- Mutation tests remove, duplicate, or replace delimiters and mutate strings, comments, integer ranges, alternation separators, and invalid characters. They assert only public document invariants and recognized results.
- Range tests include multibyte UTF-8 input and assert that every exposed range is ordered, in bounds, and safe to use for source slicing.
- Source-preservation tests assert that token iteration covers the intended complete tokenization in source order, including trivia and invalid input, without relying on copied token text.
- Public syntax snapshots use a normalized representation of token kinds, ranges, recognized language constructs, recovery regions, syntax errors, and cursor contexts. Private Rust `Debug` output is not a snapshot contract.
- A compile-time test asserts that `ParsedDocument` is `Send + Sync`.
- Existing lexer and parser snapshots are prior art for representative inputs, but their assertions should migrate upward to the public syntax seam instead of being copied as internal tests.
- Existing grammar, generator, and CLI snapshots are regression protection for the compatibility lowering. Seeded outputs remain unchanged.
- Workspace tests and documentation tests must pass after each migration slice. There must be no interval in the final ticket state where both old and new parsers remain active sources of truth.

## Out of Scope

- Implementing `bnfgen-analysis`, semantic symbol identity, name resolution, graph analysis, or `GenerationPlan`.
- Redesigning or publishing syntax or semantic diagnostics through LSP.
- Implementing the LSP server, protocol capabilities, workspace state, imports, or multi-file grammar semantics.
- Renaming or otherwise completing the generation-facing crate reorganization beyond the minimum compatibility integration.
- Removing `RawGrammar` or `CheckedGrammar`; their replacement belongs to the subsequent analysis and generation-plan work.
- Changing generator algorithms, weighted production behavior, invocation limits, seeded output, CLI commands, or MCP behavior.
- Formatting, source rewriting, refactoring, structural editing, or a public generic syntax traversal interface.
- Rowan, Tree-sitter, a handwritten replacement parser, or another parsing algorithm.
- Incremental parsing, incremental lexing, persistent subtrees, or edit-based range remapping.
- Caching line, column, or UTF-16 positions on tokens, or implementing an LSP-specific source index.
- Performance optimization beyond demonstrating that full-document parsing is acceptable for the current example corpus.
- Cross-file recovery, imports, includes, or workspace-level symbol lookup.

## Further Notes

- The domain glossary is `CONTEXT.md`. Use `Start Symbol` and `Unreachable Rule` consistently; do not introduce parser-oriented synonyms for those domain concepts.
- The accepted crate topology and ownership are recorded in `docs/adr/0002-split-syntax-analysis-and-generation-crates.md`.
- The total token-backed document decision is recorded in `docs/adr/0003-use-token-backed-tolerant-syntax-documents.md`.
- The decision to avoid Rowan and keep storage token-backed is recorded in `docs/adr/0004-keep-syntax-storage-token-backed.md`.
- The full target seam, dependency rules, testing contracts, and deliberate non-goals are in `docs/architecture/frontend.md`.
- Prior research on tolerant parsing and current implementation constraints is in `docs/research/tolerant-parsing-for-lsp.md`. Its production-tool comparisons remain useful evidence, while its superseded generic-CST alternatives are explicitly marked as historical.
- The next flow after this spec is `/to-tickets`. Tickets produced from this spec are already `ready-for-agent` and must declare their blocking edges; they do not go through triage.
