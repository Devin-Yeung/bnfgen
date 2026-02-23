use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand, Debug, Clone)]
pub enum Command {
    /// Check grammar for errors (duplicate rules, undefined symbols, dead loops, etc.)
    Check {
        #[arg(short, long)]
        /// Path to the BNF grammar file
        grammar: PathBuf,
        #[arg(long)]
        /// Check for unused rules and trap loops
        strict: bool,
        #[arg(short, long, required_if_eq("strict", "true"))]
        /// The starting non-terminal to check for unused rules (required if --strict is set)
        start: Option<String>,
    },
    /// Generate random strings from the grammar (includes checking phase)
    Gen {
        #[arg(short, long)]
        /// Path to the BNF grammar file
        grammar: PathBuf,
        #[arg(short, long, default_value = "S")]
        /// Starting non-terminal for generation
        start: String,
        #[arg(short, long, default_value = "1")]
        /// Number of strings to generate
        count: usize,
        #[arg(long)]
        /// Random seed for reproducible output
        seed: Option<u64>,
        #[arg(long, default_value = "10000")]
        /// Maximum generation steps per attempt before retrying with a fresh attempt
        max_attempts: usize,
    },
}
