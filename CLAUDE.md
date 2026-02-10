# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## What This Is

Bnfgen is a BNF grammar-based random string/fuzzy test generator written in Rust. It extends standard BNF with regex support (`re("[a-zA-Z]")`), invoke limits (`{min, max}`), weighted branches, and typed non-terminals (`<E: "int">`). It performs semantic analysis (dead loops, unreachable rules, undefined symbols, invalid ranges) with rich error diagnostics via miette.

## Build & Development Commands

```bash
cargo build                    # Build (LALRPOP runs via build.rs to generate src/parser.rs)
cargo test                     # Run all tests
cargo test --locked --all-features --all-targets  # CI-style test run
cargo test <test_name>         # Run a single test by name
cargo clippy                   # Lint
cargo fmt --check              # Check formatting
cargo fmt                      # Format code
cargo run -- -g <grammar.bnf>  # Run the CLI checker tool
```

Snapshot tests use [insta](https://insta.rs/). To update snapshots after intentional changes:
```bash
cargo insta review             # Interactive snapshot review
cargo insta accept             # Accept all pending snapshots
```

Nix-based development (optional): `devenv` provides the Rust toolchain, `cargo-deny`, `cargo-machete`, and pre-commit hooks (clippy, rustfmt, nixfmt, taplo). Formatting is orchestrated by `treefmt.toml`.

## Architecture

The pipeline follows: **Input text -> Lexer -> Parser -> RawGrammar -> CheckedGrammar -> Generator -> Output**

- **Lexer** (`src/lexer.rs`, `src/token.rs`): Tokenization via [logos](https://github.com/maciejhirsz/logos). `Token` enum defines all grammar tokens with logos derive macros. The `Lexer` wraps the logos `SpannedIter` to produce LALRPOP-compatible `(Loc, Token, Loc)` triples.

- **Parser** (`src/parser.lalrpop`): LALRPOP grammar definition. Built at compile time by `build.rs` calling `lalrpop::process_src()`, which generates `target/.../parser.rs`. Defines the grammar syntax: rules, alternatives, symbols (terminals, non-terminals, typed non-terminals, regex).

- **RawGrammar** (`src/grammar/raw.rs`): Unvalidated parse result — a `Vec<Rule>`. Entry point is `RawGrammar::parse()`. Provides validation methods: `check_undefined()`, `check_duplicate()`, `check_repeats()`. Converts to `CheckedGrammar` via `to_checked()` (which runs all checks first). Can also build a `GrammarGraph` via `.graph()` for graph-based analysis.

- **GrammarGraph** (`src/grammar/graph.rs`): petgraph `DiGraph` over grammar rules. Uses Tarjan's SCC for dead-loop detection (`check_trap_loop()`) and DFS for unreachable rule detection (`check_unused(start)`).

- **CheckedGrammar** (`src/grammar/checked.rs`): Validated grammar stored as `IndexMap<NonTerminal, WeightedProduction>`. Core method is `reduce()` which resolves a symbol: terminals pass through, non-terminals pick from candidates (exact match for typed, any matching name for untyped), regex generates via `Regex::generate()`.

- **Generator** (`src/generator.rs`): Stack-based iterative generation (`Generator`) and recursive tree generation (`TreeGenerator`). Both use `CheckedGrammar::reduce()` with a `State` that tracks RNG and invoke counts.

- **Grammar types** (`src/grammar/`): `symbol.rs` defines `SymbolKind` (Terminal, NonTerminal, Regex) and typed non-terminals. `alt.rs` defines `Alternative` with `Limit` (Unlimited, Limited{min,max}) and weight. `production.rs` defines `WeightedProduction` which selects alternatives respecting invoke limits and weights. `state.rs` tracks per-alternative invocation counts.

- **Error reporting** (`src/error.rs`, `src/report.rs`): All errors are `thiserror` + `miette::Diagnostic` for rich source-annotated diagnostics. `Reporter` collects multiple diagnostics and renders them.

- **Regex** (`src/regex.rs`): Parses regex via `regex_syntax` into HIR, then generates random matching strings. Avoids generating strings that collide with existing grammar terminals.

- **CLI** (`src/bin/cli/`): clap-based CLI that parses a grammar file and runs semantic checks (not generation). Accepts `-g <file>` and optional `--check-unused <start_rule>`.

## Key Patterns

- The LALRPOP parser uses an external lexer (logos-based) rather than LALRPOP's built-in lexer. The `extern` block in `parser.lalrpop` bridges them.
- Snapshot testing with insta is used extensively. Snapshots live in `src/snapshots/` and `src/grammar/snapshots/`.
- `Span` is the project's source span type, convertible from `Range<usize>` and used throughout for error reporting positions.
- `Rc<String>` is used for terminal values to enable cheap cloning during generation.
