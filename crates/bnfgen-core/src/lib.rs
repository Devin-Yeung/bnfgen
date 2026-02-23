//! # Bnfgen
//!
//! BNF grammar-based random string/fuzzy test generator.
//!
//! Bnfgen extends standard BNF with several powerful features:
//!
//! - **Regex support**: Generate strings matching patterns using `re("[a-zA-Z]")`
//! - **Invoke limits**: Control expansion with `{min,max}` syntax
//! - **Weighted branches**: Assign probabilities to alternatives
//! - **Typed non-terminals**: Use `<E: "int">` for type-safe grammar rules
//!
//! ## Quick Start
//!
//! ```rust
//! use bnfgen::{RawGrammar, Generator, Result};
//!
//! fn main() -> Result<()> {
//!     // Parse a grammar from a string
//!     let grammar = RawGrammar::parse(
//!         r#"
//!         <S> ::= "hello" | "world" ;
//!         "#
//!     )?;
//!
//!     // Validate and convert to CheckedGrammar
//!     let checked = grammar.to_checked()?;
//!
//!     // Create a generator
//!     let mut gen = Generator::new(checked);
//!
//!     // Generate a random string
//!     let output = gen.generate("S", &mut rand::rng())?;
//!     println!("{}", output);
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Grammar Syntax
//!
//! Basic BNF syntax:
//! ```text
//! <S> ::= "hello" ;
//! <E> ::= <E> "+" <E> | <E> "*" <E> | "1" | "2" ;
//! ```
//!
//! With regex:
//! ```text
//! <Word> ::= re("[a-zA-Z]+") ;
//! ```
//!
//! With invoke limits (repeat counts):
//! ```text
//! <List> ::= <Item> {1, 10} ;  // Repeat 1 to 10 times
//! ```
//!
//! Typed non-terminals (for type-safe polymorphism):
//! ```text
//! <E: "int"> ::= "1" | <E: "int"> "+" <E: "int"> ;
//! <E: "str"> ::= "\"a\"" | <E: "str"> "+" <E: "str"> ;
//! ```
//!
//! ## Validation
//!
//! `RawGrammar::to_checked()` performs comprehensive semantic analysis:
//!
//! - Detects undefined non-terminals
//! - Validates invoke limits (min <= max)
//! - Optionally checks for unreachable rules via `GrammarGraph::check_unused()`
//! - Optionally detects trap loops via `GrammarGraph::check_trap_loop()`
//!
//! ## Error Reporting
//!
//! Errors use the [miette] library for rich diagnostics:
//!
//! ```rust
//! use bnfgen::{RawGrammar, Reporter, Style};
//!
//! let grammar = RawGrammar::parse("<E> ::= <Undefined>;").unwrap();
//! match grammar.to_checked() {
//!     Ok(_) => println!("Valid"),
//!     Err(e) => {
//!         let mut reporter = Reporter::new(Style::NoColor);
//!         reporter.push(e);
//!         eprintln!("{}", reporter.report_to_string());
//!     }
//! }
//! ```
//!
//! [miette]: https://docs.rs/miette

pub mod error;
pub mod generator;
pub mod grammar;
mod lexer;
pub mod parse_tree;
mod regex;
pub mod report;
mod span;
mod token;
mod utils;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(parser);

// Re-exports for convenience

pub use grammar::checked::CheckedGrammar;
pub use grammar::graph::GrammarGraph;
/// Core grammar types
pub use grammar::raw::RawGrammar;

/// Generators
pub use generator::{Generator, TreeGenerator};

/// Error handling
pub use error::{Error, Result};

/// Symbol types (for advanced use)
pub use grammar::symbol::{NonTerminal, SymbolKind, Terminal, Ty};

/// Reporting
pub use report::{Reporter, Style};

/// Type alias for convenience - `Grammar` is the most common entry point
pub use RawGrammar as Grammar;
