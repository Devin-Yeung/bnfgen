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
        /// Check for unreachable rules (need to give the starting rule)
        check_unused: Option<String>,
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
    },
}
