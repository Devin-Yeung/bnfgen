# Token-backed tolerant syntax documents

`bnfgen-syntax` parses source text into a `ParsedDocument`: a lossless, total snapshot of one source text that stays useful while the grammar is being edited.

## Decision

- **Lossless.** The document retains the original source and a complete, source-ordered token buffer. The buffer includes significant tokens, trivia, invalid input, and unterminated strings. A token is a kind plus a UTF-8 byte range. Token text is always sliced from the source; nothing is copied or decoded during lexing or parsing.
- **Total.** Every `&str` produces a `ParsedDocument`. There is no failed-parse or empty-document outcome. Problems are data: syntax diagnostics and recovery observations.
- **Token-backed storage.** Recognized syntax is held in private records that refer to tokens or byte ranges. No generic concrete syntax tree is built.
- **Typed public views.** Callers use immutable, language-specific views (documents, rules, alternatives, symbols, non-terminals, and so on) that borrow from the document. A required child that the source does not establish is absent. The syntax stage never fabricates a token, a semantic value, or a complete generation object.
- **Single-line strings.** A string literal cannot span a line break. An unclosed quote ends at its line, so it cannot swallow later rules. The same rule applies to the legacy lexer until it is removed.
- **One parser.** A single private, handwritten recursive-descent parser runs over the significant (non-trivia) token stream. Trivia stays in the buffer and is never reinserted after parsing.
- **Missing syntax** produces a diagnostic at the insertion point, which is an empty range where the token would go. It does not consume or invent a token.
- **Unexpected syntax** is consumed into the skipped list of the nearest construct that is still being parsed. Those ranges are recovery observations. Diagnostics and recovery observations are separate public data.
- **Parse errors and lexical errors** are merged into one list ordered by source position.
- **Synchronization.** A following rule ends the current rule only after lookahead confirms a complete rule header: `<`, identifier, optional `:` and a terminated string, `>`, then `::=`. A bare `<` and identifier stays with the enclosing construct and may become a partial non-terminal reference. Each construct stops at its own set of synchronization tokens (`>`, `}`, `)`, `|`, `;`). Every recovery loop must consume input, return to its caller, or record missing syntax, and a loop that does none of these is a bug.
- **Diagnostics are the correctness signal.** Recovery results are always returned, and the syntax layer does not flag or reject them. Every recovery observation is covered by at least one syntax diagnostic, and tests enforce this for every prefix of every fixture. A consumer that needs correct structure checks the diagnostics whose ranges fall inside the construct it uses. The syntax layer exposes no notion of "complete" or "trusted".
- **Trust boundary.** Analysis establishes semantic identities and graph edges only from constructs with no syntax diagnostics in their range. Strict lowering requires a document with no syntax diagnostics. Editor queries may use recovered results.
- **Ranges** are UTF-8 byte offsets. Line, column, and UTF-16 positions belong to protocol adapters, not to tokens or views.
- **Performance model.** Each edit reparses the whole document. Incremental lexing, incremental parsing, and persistent subtree reuse are out of scope until measurements justify them.
- **Regression contract.** Normalized snapshots of public results (tokens, views, diagnostics, recovery observations, cursor contexts) are the contract. Private Rust `Debug` output is not.

## Consequences

- Downstream code cannot observe parser internals, so recovery can change without breaking callers.
- A change to how valid grammars are read requires explicit approval, because the generation output for those grammars must stay stable.
- Recovery is tested through public behavior: prefix tests of representative grammars, mutation tests, range-invariant tests, and snapshot fixtures for incomplete references at end of input, before a later rule, and before typed and untyped rule headers.
