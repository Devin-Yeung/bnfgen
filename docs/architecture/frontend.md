# Front-end architecture

This document records the target crate seams for the parser and front-end refactor. The refactor prepares Bnfgen for a future language server, but does not design or implement LSP capabilities, workspace behavior, or protocol handling.

The governing decisions are [ADR 0001](../adr/0001-separate-analysis-from-generation-planning.md), [ADR 0002](../adr/0002-split-syntax-analysis-and-generation-crates.md), [ADR 0003](../adr/0003-use-token-backed-tolerant-syntax-documents.md), and [ADR 0004](../adr/0004-keep-syntax-storage-token-backed.md).

## Dependency direction

```text
bnfgen-syntax
      |
      v
bnfgen-analysis
      |
      v
    bnfgen
      |
      v
  bnfgen-cli

Future only:

bnfgen-lsp -> bnfgen-syntax
bnfgen-lsp -> bnfgen-analysis
```

Dependencies only point downward in this diagram. Lower crates know nothing about their adapters or consumers.

## Crate responsibilities

### `bnfgen-syntax`

Owns the source language and the tolerant parsing interface:

- source text and byte ranges;
- raw tokens, trivia, and lexical errors;
- the Logos lexer and LALRPOP grammar;
- parser recovery;
- `ParsedDocument`, language-specific syntax views, and syntax errors.

The core output is a token-backed `ParsedDocument`. It preserves the original source, raw tokens, trivia, invalid input, recovered regions, and syntax errors. LALRPOP consumes a significant-token adapter and contributes structural ranges and recovery events; its recovery types are implementation details and do not cross the crate interface. Typed syntax access is a view over the retained source model, not a strict generation AST.

Incomplete syntax must not discard later valid rules or unrecognized text. Syntax parsing does not decode string or integer values, compile regular expressions, resolve names, or establish generation invariants.

The public contract follows these rules:

- `parse(source)` is total and returns a `ParsedDocument` for every input; syntax errors live in the document;
- token and recognized-syntax storage is private behind immutable views, iterators, ranges, and source-slicing operations;
- a token stores its kind and UTF-8 byte range, while its raw text is sliced from the document source;
- line, column, and UTF-16 positions are derived by a separate source index or adapter, not cached on every token;
- `ParsedDocument` is `Send + Sync`, so an LSP may retain and query documents across worker threads.

The complete source-backed token buffer is the lossless representation. Private records identify recognized rules, alternatives, symbols, and recovery regions by token identity or byte range; they do not materialize a second generic tree. Required syntax that is absent simply has no record. Typed wrappers such as rule and non-terminal views provide domain-specific accessors and return `None` when incomplete syntax omits a child.

The public interface provides the capabilities callers need rather than exposing storage: source and token iteration, rules and their typed children, syntax errors, token lookup at a byte offset, and syntax-context classification at a byte offset. `bnfgen-analysis` consumes these language-specific views and never matches raw token kinds to reconstruct the grammar. Internal `TokenId` values, LALRPOP recovery types, significant-token adapters, and storage records do not cross the crate interface.

This crate does not depend on Rowan, Miette, Petgraph, Rand, generation code, CLI code, or LSP types. It does not decode literal values, resolve names, compile regular expressions for generation, or decide whether a grammar can execute.

### `bnfgen-analysis`

Owns the meaning derived from a parsed document:

- non-terminal identities, including families and typed variants once their exact semantics are settled;
- definition and reference resolution;
- duplicate, undefined, repeat-range, and regular-expression analysis;
- the internal rule-dependency graph;
- reachability and productivity analysis;
- transport-neutral structured diagnostics.

Petgraph may be used internally, but graph nodes, indices, and traversal operations are not part of the public interface. Callers consume semantic queries and diagnostics instead.

This crate depends on `bnfgen-syntax`. It does not depend on Miette, Rand, CLI code, or LSP types.

### `bnfgen`

Remains the user-facing generation library:

- start-specific `GenerationPlan` construction;
- weighted productions and invocation state;
- string and parse-tree generators;
- generation-time errors.

It depends on `bnfgen-analysis`. It may re-export common syntax and analysis entry points for ergonomic library use, while lower-level consumers remain free to depend on the narrower crates directly.

The current `crates/bnfgen-core` directory should become `crates/bnfgen`; the word `core` no longer names a coherent responsibility.

### `bnfgen-cli`

Owns adapters and process orchestration:

- command-line arguments and file IO;
- Miette rendering of structured diagnostics;
- `check` and `gen` command behavior;
- MCP transport and request adaptation.

Parsing, semantic checks, graph construction, and generation policy do not belong in the CLI crate.

### Future `bnfgen-lsp`

The LSP crate is not created during the parser refactor. When implemented, it will depend directly on `bnfgen-syntax` and `bnfgen-analysis`. It will depend on `bnfgen` only if editor-side generation becomes an explicit feature.

## Data flow

```text
source text
    |
    v
token-backed ParsedDocument        bnfgen-syntax
    |
    v
Analysis + structured diagnostics  bnfgen-analysis
    |
    v
GenerationPlan(Start Symbol)       bnfgen
    |
    v
generated string or parse tree     bnfgen
```

`ParsedDocument` may contain recovered or missing syntax. Analysis is best-effort and retains the whole document. A `GenerationPlan` is created only when the selected Start Symbol's executable subgraph satisfies the required generation invariants.

## Testing contracts

| Crate | Observable snapshot contract |
| --- | --- |
| `bnfgen-syntax` | complete tokens, recognized syntax, recovery regions, cursor contexts, and selected typing states |
| `bnfgen-analysis` | normalized symbol information and structured diagnostics |
| `bnfgen` | existing seeded generation and parse-tree outputs |
| `bnfgen-cli` | rendered diagnostics, CLI behavior, and MCP outputs |

Existing output-oriented snapshots should be reused wherever behavior is unchanged. Syntax snapshots use a normalized representation; private Rust `Debug` layouts are not stable contracts. Prefix and mutation tests complement snapshots by asserting that incomplete input never panics and all reported ranges remain valid.

## Deliberate non-goals for this refactor

- LSP protocol or capability design;
- an empty `bnfgen-lsp` placeholder crate;
- workspace or import semantics;
- incremental parsing;
- separate parser, graph, or diagnostics crates;
- replacing LALRPOP with another parsing algorithm.
