# Parser output and recovery designs: comparison for `bnfgen-syntax`

**Question.** Our syntax crate currently stores recognized facts, diagnostics, and recovery in separate places, and the storage shape is not settled. Which production design gives a single source of truth for recovery, and what should `bnfgen-syntax` adopt?

**Scope.** Three designs: rust-analyzer's event-based parser (as described by matklad), Ruff's Python parser, and OXC's JavaScript/TypeScript parser. Each claim cites the pinned source revision. The existing note [`tolerant-parsing-practice.md`](tolerant-parsing-practice.md) already covers rust-analyzer, Tree-sitter, and Lezer in more depth; this note does not repeat that.

**Method.** I read the primary sources listed below directly (no secondary write-ups). Revisions: OXC `00484e16`, Ruff `8d122c1f` (Ruff 0.17.1 crate), and matklad's article dated 2023-05-21.

## Executive summary

1. **All three keep errors out of the recognized structure's identity.** None stores a "has recovery" flag on a node as the source of truth. Ruff and OXC keep one flat error list and derive everything else; rust-analyzer uses `ERROR` nodes in the one tree.
2. **The strongest single-source pattern is Ruff's:** the typed syntax structure is always produced and contains explicit placeholder nodes for invalid or missing input. Errors live in a flat list that is merged by source position. Recovery policy is list-level, not per-token.
3. **Lexer errors are tokens that the parser sees.** Ruff emits an `Unknown` token for unrecognised input, and OXC emits `Undetermined`. Our `Invalid` kind is excluded from the significant stream, which is an outlier among these three.
4. **Unterminated strings end at the line.** Both OXC and Ruff stop a single-quoted string at `\r` or `\n`. Bnfgen does the same: a string literal cannot span a line break.
5. **Our current design has duplicated facts.** An unexpected token is recorded three times (a diagnostic, a recovery observation, and an `Error` node in the private tree). The fix is to make one private model the source of truth and derive the public views, diagnostics, and recovery observations from it.

## rust-analyzer / matklad: event-based recognition, tree output

Source: matklad, [*Resilient LL Parsing Tutorial*](https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html), and rust-analyzer's parser ([`parser.rs`](https://github.com/rust-lang/rust-analyzer/blob/e96ea7a5805992ec868e8ab185bc371267048421/crates/parser/src/parser.rs#L258-L307), cited in the existing note).

- The parser emits a flat list of events (`Open`, `Close`, `Advance`). A separate step turns the events into a tree. The article keeps the grammar free of tree-building code.
- Errors are a side effect of parsing. The article says "errors can be considered to be a side-effect of the parsing process" and stores them outside the tree; `advance_with_error` wraps an unexpected token in an `ErrorTree` node.
- Recovery uses recovery sets. `advance_with_error` consumes an unexpected token only when the current token is not in the recovery set; the article suggests bitsets in the real implementation.
- Progress: "if a loop tries to parse an item, item parsing must consume at least one token (if only to report an error)". The article also adds a fuel counter to catch non-progressing loops.
- Trivia is attached by the tree sink, not seen by the grammar.

Relevance: the event pipeline is useful, but the article's *output* is a homogeneous tree with `ERROR` nodes. `bnfgen-syntax` does not use a tree output.

## Ruff: typed AST with placeholders, flat merged errors, list-level recovery

Sources (`astral-sh/ruff@8d122c1f`):

- Output shape. [`parse_unchecked`](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/lib.rs#L289) returns `Parsed<T>`, which holds the AST, the `Tokens`, and `errors` ([`lib.rs` L395–L439](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/lib.rs#L395-L439)). The AST is always present; there is no failed-parse variant.
- Errors merged by position. [`finish`](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/parser/mod.rs#L249-L300) merges parse errors and lexer errors into one list in source order. When both are at the same start, the parse error is dropped in favour of the lexer error.
- Placeholders in the AST. A `MatchAs` with both a pattern and a name has no valid expression form, so the conversion produces `ExprName { id: Name::empty(), ctx: ExprContext::Invalid }` ([`parser/recovery.rs`](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/parser/recovery.rs#L150-L170)). Invalid structure stays in the one tree with a marker, rather than being removed.
- Zero-width missing ranges. `missing_node_range` is `TextRange::empty(self.prev_token_end)`, a zero-width range at the end of the previous token ([`parser/mod.rs` L367–L380](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/parser/mod.rs#L367-L380)). This is our "missing slot" idea.
- Error dedup. `add_error` skips an error whose start equals the previous error's start ([`parser/mod.rs` L658–L675](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/parser/mod.rs#L658-L675)). That prevents cascades at a single location.
- List recovery. Each list parser keeps a `RecoveryContext`, which is the union of the enclosing list kinds ([`parser/mod.rs` L725–L774](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/parser/mod.rs#L725-L774)). When the current token is neither an element nor a terminator, the parser checks whether an enclosing context can take it; if not, it reports and consumes. This is the recovery set from matklad, expressed as a stack of contexts.
- Progress. `ParserProgress::assert_progressing` panics if a list iteration did not advance the token ID ([`parser/progress.rs`](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/parser/progress.rs)).
- Lexer errors become tokens. [`lexer.rs` L134–L147](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/lexer.rs#L134-L147) returns a `TokenKind::Unknown` token for unmatched input, which the parser then sees.
- Unclosed strings end at the line. For non-triple-quoted strings, a `\r` or `\n` ends the token with `UnclosedStringError` and a flag, and the lexer stops there ([`lexer.rs` L1010–L1020](https://github.com/astral-sh/ruff/blob/8d122c1fd2b8f16d494ff8a8f8ee6f264bba7ceb/crates/ruff_python_parser/src/lexer.rs#L1010-L1020)).

## OXC: typed AST, fatal-error channel, lexer tokens that the parser sees

Sources (`oxc-project/oxc@00484e16`):

- Output contract. [`ParserReturn`](https://github.com/oxc-project/oxc/blob/00484e165da4b48cd8631e17e39ca9a25403d62b/crates/oxc_parser/src/lib.rs#L165-L205) has `program`, `diagnostics`, and `panicked`/`fatal_error`. Recoverable errors leave a full AST with `fatal_error == false`. A fatal error leaves an empty program. The docs say "the AST may be present and semantically invalid" and that semantic checks run in a later stage ([`lib.rs` L176–L191](https://github.com/oxc-project/oxc/blob/00484e165da4b48cd8631e17e39ca9a25403d62b/crates/oxc_parser/src/lib.rs#L176-L191)).
- Fatal versus recoverable. `set_unexpected` turns the first unexpected token into a fatal error and stops (`error_handler.rs`). Recoverable errors go into `self.errors` with `error(...)` and parsing continues ([`error_handler.rs`](https://github.com/oxc-project/oxc/blob/00484e165da4b48cd8631e17e39ca9a25403d62b/crates/oxc_parser/src/error_handler.rs)).
- Lexer errors become tokens. An unterminated string returns `Kind::Undetermined` and pushes a lexer diagnostic ([`lexer/string.rs` L36–L47](https://github.com/oxc-project/oxc/blob/00484e165da4b48cd8631e17e39ca9a25403d62b/crates/oxc_parser/src/lexer/string.rs#L36-L47)). The parser's `set_unexpected` pops that diagnostic when it sees `Undetermined` or `Eof`, so the lexer's message wins.
- Strings end at the line. The byte-search table for string literals stops at `\r` or `\n` ([`lexer/string.rs` L15](https://github.com/oxc-project/oxc/blob/00484e165da4b48cd8631e17e39ca9a25403d62b/crates/oxc_parser/src/lexer/string.rs#L15)), and reaching one raises the unterminated-string error.
- Speculative parsing. `checkpoint` and `checkpoint_with_error_recovery` snapshot the lexer and error state so TypeScript-style lookahead can roll back ([`cursor.rs` L315–L345](https://github.com/oxc-project/oxc/blob/00484e165da4b48cd8631e17e39ca9a25403d62b/crates/oxc_parser/src/cursor.rs#L315-L345)). Our lookahead is a plain index check, so this is not needed yet.

## Comparison

| Concern | Current `bnfgen-syntax` (after the refactor) | rust-analyzer / matklad | Ruff | OXC |
| --- | --- | --- | --- | --- |
| Structure output | Private records, plus a generic `sink` tree that is discarded | One lossless untyped tree, typed wrappers on top | Typed AST, always complete | Typed AST, empty on fatal error |
| Unexpected input | A diagnostic, a recovery observation, and an `Error` node | `ERROR` node in the tree; error side effect | Node placeholder where possible; error in flat list | Diagnostic; fatal stop when unrecoverable |
| Missing input | Diagnostic with empty range; absent `Option` child | `MISSING` or error side effect | Zero-width placeholder at previous token end | Dummy node or error |
| Recovery state | Overlap query against a list | Structural (`ERROR` in subtree) | Placeholder markers and error list | Error list; placeholder dummies |
| Lexer errors | Excluded from significant stream (`Invalid`) | Error token in stream | `Unknown` token the parser sees | `Undetermined` token the parser sees |
| Unclosed string | Ends at the line | Not applicable (language-specific) | Ends at the line | Ends at the line |
| Error merge and order | Lexer first, then parser, by push order | Errors as recorded | Merged by position | Ordered list with lexer messages substituted |
| Loop progress | Ad hoc; `fuel` in the parser | Rule: consume or report | Progress assertion per list | Checkpoint and fatal stop |

## What this means for `bnfgen-syntax`

The three designs agree on five points, and our current code breaks four of them:

1. Every recognized construct and every missing or unexpected source fact is recorded once, where it is owned. Ruff and rust-analyzer both do this. We record unexpected tokens in three places.
2. Lexer failures are visible to the parser as tokens. We hide `Invalid` from the parser, so a rule containing invalid input has no marker where the invalid input sits.
3. Single-line strings. Both production lexers end a string at the line, and so does Bnfgen.
4. Progress and recovery are list-level policies with explicit sets, as in Ruff and matklad. Ours are partly local and partly implicit.

The point where the three designs differ, and where we need to decide, is the **output shape**:

- *Tree output* (rust-analyzer): not used by `bnfgen-syntax`.
- *Typed AST with placeholders* (Ruff, OXC): used by `bnfgen-syntax` as a private syntax model, not a generation AST, with typed views as the public seam over it.

## Proposed single source of truth

Replace the current pipeline (parser → events → generic `sink` tree → `lower` → records, plus separate diagnostic and recovery vectors) with one private model:

- **One model, typed per construct.** A rule record has slots for its lhs, `::=`, alternatives, and `;`. Each slot is `Present(range)` or `Missing(insertion_point)`. This is the Ruff placeholder idea in our token-backed form. The model stores no node kinds that do not correspond to a grammar construct.
- **Recovery is owned by the construct.** Each construct has a `skipped: Vec<(Range, SkipKind)>` list of source it consumed without structurally attaching it. The parser appends to the nearest construct that is still being parsed. Whether a construct has recovery is then a method that checks its own list and its children's (structural), with no overlap query and no stored flag.
- **Diagnostics are derived.** `diagnostics()` is a projection: missing slots, skipped ranges, and lexer errors, merged by position with duplicate-start suppression (Ruff). It is no longer pushed by the parser, so the three-way duplication goes away.
- **Recovery observations are derived.** The public `recovery()` is the skipped lists flattened and sorted, with lexer recovery tokens included.
- **Lexer decisions are explicit in the lexer.** Invalid input becomes a token kind the parser can see (`Unknown`, as in Ruff). Single-line strings end at the line, as in both production lexers. Whether either changes the legacy behaviour for valid grammars must be checked against the legacy lexer and fixtures; I have not done that check.
- **Recovery policy.** Each list parser takes a recovery set (`;`, `>`, `}`, `)`, `|`, and a confirmed rule header, per ADR 0002), keeps a context union in the style of Ruff, and asserts progress per iteration.

Consequences for the current code: `parser/sink.rs` and the tree-based `lower.rs` go away. The private records become the model, built directly by the parser. Snapshot output changes in two places that need your decision (see below).

## Open decisions for you

1. **Diagnostic order.** Today lexer diagnostics come first. The merged, position-sorted order used by Ruff is easier to review and matches what an editor shows. This changes the existing snapshots for fixtures with lexer errors. Do you want position order?
2. **Lexer string rule.** Decided: strings are single-line in both the syntax crate and the legacy lexer.
3. **Invalid input as a token.** Making `Invalid` significant is consistent with Ruff and OXC. The alternative is to keep the current exclusion and accept that invalid input is only visible through diagnostics.

## Sources

- matklad, *Resilient LL Parsing Tutorial*, 2023-05-21: https://matklad.github.io/2023/05/21/resilient-ll-parsing-tutorial.html
- Ruff (`8d122c1f`, crate version 0.17.1):
  - `crates/ruff_python_parser/README.md`
  - `crates/ruff_python_parser/src/lib.rs` (`parse_unchecked`, `Parsed`, error merge)
  - `crates/ruff_python_parser/src/parser/mod.rs` (`expect`, `add_error`, `missing_node_range`, `parse_list`, `RecoveryContextKind`)
  - `crates/ruff_python_parser/src/parser/progress.rs`
  - `crates/ruff_python_parser/src/parser/recovery.rs`
  - `crates/ruff_python_parser/src/lexer.rs` (unknown token, unclosed string)
- OXC (`00484e16`):
  - `crates/oxc_parser/README.md`
  - `crates/oxc_parser/src/lib.rs` (`ParserReturn`, error docs)
  - `crates/oxc_parser/src/error_handler.rs`
  - `crates/oxc_parser/src/lexer/string.rs`
  - `crates/oxc_parser/src/lexer/kind.rs`
  - `crates/oxc_parser/src/cursor.rs` (checkpoints)
- Existing repo notes: [`tolerant-parsing-practice.md`](tolerant-parsing-practice.md) (rust-analyzer, Tree-sitter, Lezer); ADR 0002.

## Not verified

- Ruff's `Tokens` and trivia handling: I did not check whether comments are present in the token list.
- OXC's placeholder representation for invalid expressions: I looked at the error and checkpoint paths but not at how the AST encodes an invalid node.
- The current rust-analyzer source: the existing note's citations were not re-fetched.
