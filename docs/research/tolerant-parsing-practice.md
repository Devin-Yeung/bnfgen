# Tolerant parsing in mature editor tooling

**Question.** How do mature parsers support live editing, where the input is routinely incomplete or malformed, without requiring a second parser for partial syntax?

**Scope.** This note compares the primary implementations and documentation of rust-analyzer, Tree-sitter, and Lezer. Source links below are pinned to the revisions inspected on 2026-08-30.

## Executive summary

There is no single universal parsing algorithm. The common practice is instead:

1. **Parsing is a total editor operation.** It produces syntax plus errors; it does not make the document disappear because one construct is malformed.
2. **The parser has one authoritative source-language implementation.** Recovery is part of that implementation, not a second “partial grammar” that re-recognizes failed ranges.
3. **Malformed and missing syntax is represented explicitly.** Mature systems distinguish text that was present but could not be placed (`ERROR`) from syntax that the parser inferred was missing (`MISSING` / zero-width error placeholder).
4. **Recovery preserves progress locally.** It either consumes unexpected input into an error region or leaves a known synchronization token for the enclosing parser. Parsers must guarantee progress and bound speculative recovery.
5. **Incrementality is separate from tolerance.** A parser can be fully tolerant while reparsing the whole document after every edit. Incremental reparsing/reuse is a later performance optimization with carefully restricted applicability.

For Bnfgen's small grammar and requirements for exact partial facts, the closest practical model is rust-analyzer's: one handwritten recursive-descent parser over a lossless token stream, emitting errors and partial structure as it proceeds. That does **not** require adopting a generic CST or Rowan; the existing token-backed `ParsedDocument` and private range records can remain the output representation.

## rust-analyzer: handwritten recursive descent, event output, explicit local recovery

rust-analyzer's parser grammar is handwritten Rust: its own module documentation says each grammar function corresponds to a formal production. The parser does not directly construct its final syntax representation. Instead it emits start-node, finish-node, token, and error events, which a later event processor turns into output.

- Grammar organization: [`crates/parser/src/grammar.rs`](https://github.com/rust-lang/rust-analyzer/blob/e96ea7a5805992ec868e8ab185bc371267048421/crates/parser/src/grammar.rs)
- Event representation and processing: [`crates/parser/src/event.rs`](https://github.com/rust-lang/rust-analyzer/blob/e96ea7a5805992ec868e8ab185bc371267048421/crates/parser/src/event.rs#L70-L146)
- Top-level parse always finishes its parser and processes events: [`crates/parser/src/lib.rs`](https://github.com/rust-lang/rust-analyzer/blob/e96ea7a5805992ec868e8ab185bc371267048421/crates/parser/src/lib.rs#L112-L144)

Its three small parser primitives capture the recovery policy:

- `expect(kind)` records an error when a token is absent but does **not** consume unrelated input. This represents a missing expected element.
- `err_and_bump(message)` records an error, consumes one unexpected token, and wraps it in an `ERROR` node. This guarantees forward progress.
- `err_recover(message, recovery_set)` records an error but deliberately does not consume if the current token belongs to the caller-provided recovery set; otherwise it consumes one token into `ERROR`. It treats braces specially so enclosing block parsers can retain ownership of delimiters.

Source: [`Parser::{expect, err_and_bump, err_recover}`](https://github.com/rust-lang/rust-analyzer/blob/e96ea7a5805992ec868e8ab185bc371267048421/crates/parser/src/parser.rs#L258-L307).

The important point is not the event tree specifically. It is that **one parser owns normal parsing, missing-token reporting, bad-token ownership, and synchronization sets**. Grammar productions call these primitives at the syntactic level that understands the construct; no fallback parser needs to infer partial facts after a failed parse.

rust-analyzer also shows that “editor parser” and “incremental parser” are separate concerns. Its incremental implementation first tries a safe single-token replacement, then a safely delimited braced-block reparse, and otherwise falls back rather than forcing local reuse. Source: [`crates/syntax/src/parsing/reparsing.rs`](https://github.com/rust-lang/rust-analyzer/blob/e96ea7a5805992ec868e8ab185bc371267048421/crates/syntax/src/parsing/reparsing.rs#L1-L7) and its guarded block-reparse path ([lines 77–105](https://github.com/rust-lang/rust-analyzer/blob/e96ea7a5805992ec868e8ab185bc371267048421/crates/syntax/src/parsing/reparsing.rs#L77-L105)).

## Tree-sitter: generated GLR-style parser with error-cost recovery in its one tree

Tree-sitter is the counterexample to “mature tolerance implies handwritten recursive descent.” It uses generated parsing tables and has built-in recovery. But it still follows the same architecture rule: the resulting syntax tree itself records recovery; it does not call a second partial grammar after failure.

Tree-sitter exposes two distinct recovery results:

- An `ERROR` node represents source text the parser did not recognize or place.
- A `MISSING` node represents a zero-width token inserted by recovery, but only when that recovered tree has the lowest error cost.

The official documentation describes and demonstrates both kinds: [Tree-sitter Query Syntax — `ERROR` and `MISSING`](https://tree-sitter.github.io/tree-sitter/using-parsers/queries/1-syntax.html#the-error-node). The runtime exposes `ts_node_is_error`, `ts_node_is_missing`, and `ts_node_has_error`; see [`lib/src/node.c`](https://github.com/tree-sitter/tree-sitter/blob/16d5e9d2275477ff568ff54a42dd9c0901a17728/lib/src/node.c#L512-L528).

The design trade-off is that Tree-sitter's recovery is sophisticated generic machinery and its tree is the central storage model. It is attractive when its ecosystem, grammar tooling, queries, and incremental tree reuse are desired. It is not a reason to combine a strict LALR parser with a separate hand-maintained prefix grammar.

## Lezer: generated LR parser, bounded scored recovery, explicit error nodes

Lezer is another mature generated-parser approach. Its recovery is implemented inside the parser runtime, not in a second grammar:

- On getting stuck, it tries forced reductions, a bounded number of conceptual missing-node insertions, and deletion of unexpected input; it maintains and prunes recovery stacks by score. Source: [`src/parse.ts`, `runRecovery`](https://github.com/lezer-parser/lr/blob/ed59b8b9c0c26164d6483f4c881a8c200184894e/src/parse.ts#L456-L504).
- A deleted token is stored with an error node and parser position advances. Source: [`recoverByDelete`](https://github.com/lezer-parser/lr/blob/ed59b8b9c0c26164d6483f4c881a8c200184894e/src/stack.ts#L256-L269).
- Inserted recovery alternatives get a zero-width error node and incur a recovery penalty; the number and depth of alternatives are bounded. Source: [`recoverByInsert`](https://github.com/lezer-parser/lr/blob/ed59b8b9c0c26164d6483f4c881a8c200184894e/src/stack.ts#L279-L314).

This is a viable alternative if Bnfgen deliberately chooses to invest in a generated parser runtime with generic error-recovery semantics. It is substantially more machinery than Bnfgen needs, and the current LALRPOP setup does not provide this integrated scored-recovery model.

## Implications for Bnfgen

Bnfgen already has the right external contract: `parse(source) -> ParsedDocument`, a complete source-backed token buffer, retained errors, recovery ranges, and partial typed views. The issue is internal duplication:

```text
current
  strict LALRPOP parser
    -> identifies a failed range
    -> second LALRPOP prefix parser recognizes facts in that range
```

That is not the normal end-state represented by the systems above. It separates complete syntax and typing-state syntax into two source-language grammars that must evolve together.

### Recommended implementation direction

Keep:

- the Logos lexer and complete source-ordered token buffer;
- the total `ParsedDocument` interface;
- byte-range-backed records and language-specific immutable views;
- full-document reparse as V1.

Replace the two LALRPOP grammars with one private handwritten parser module that consumes the significant-token view and directly produces records, errors, and recovery ranges. Its internal interface can be narrow, for example:

```rust
fn parse_document(tokens: &[SignificantToken]) -> ParseOutput;
```

The parser should provide named primitives analogous to rust-analyzer's:

- `at` / `bump` for token navigation;
- `expect` for absent expected syntax without swallowing a plausible next construct;
- `error_and_bump` for unexpected syntax, with an unconditional progress guarantee;
- `recover_until` / construct-specific synchronization sets;
- construct parsers such as `parse_rule`, `parse_non_terminal`, `parse_alternative`, `parse_symbol`, and `parse_repeat` that return partial records whose absent children are `None`.

For Bnfgen specifically, useful synchronization points are `;` (rule terminator) and a trustworthy next-rule start (a `<` which can begin a rule declaration). Nested parsers should preserve their closing delimiters (`>`, `}`, `)`) for the parser that owns them, just as rust-analyzer recovery sets preserve enclosing delimiters.

### Required invariants and tests

The implementation should make these recovery guarantees explicit and testable:

1. Every parser loop either consumes a token, returns, or reports a missing token without retrying the same state indefinitely.
2. Every input returns a document, and all reported/public ranges are UTF-8 boundaries within source.
3. Tokens not structurally recognized remain in the token buffer and are covered by an error/recovery fact rather than silently disappearing.
4. A malformed rule cannot consume a following valid rule.
5. Incomplete constructs retain only facts actually spelled in source; a missing delimiter is not fabricated as a real token or complete generation object.
6. Valid documents preserve the existing recognized structure and downstream generation behavior.
7. Prefix and mutation tests exercise ordinary editing states, not only curated malformed examples.

## Decision framing

The project currently records “do not replace LALRPOP with another parsing algorithm” as a non-goal in the syntax-rewrite spec and front-end architecture document. Adopting the recommendation above therefore requires reopening that decision explicitly, rather than silently changing implementation underneath it.

The decision is not “handwritten parsers are universally better.” It is:

> For Bnfgen's small grammar and its requirement to expose precise partial facts during ordinary typing, one parser with explicit grammar-local recovery has lower long-term duplication and clearer ownership than two coordinated LALRPOP grammars.

If the project instead wants to retain a generated parser, align with Tree-sitter/Lezer by choosing a parser system whose **single parse result** represents error and missing syntax. Do not extend the current two-grammar fallback pattern as the long-term design.
