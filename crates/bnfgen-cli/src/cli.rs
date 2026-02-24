use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Clone, ValueEnum)]
pub enum TransportType {
    Stdio,
    #[value(name = "http")]
    StreamableHttp,
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
        #[arg(long)]
        /// Maximum generation steps per attempt before retrying with a fresh attempt
        max_steps: Option<usize>,
        /// Maximum generation attempts before giving up (default: 100)
        #[arg(long, default_value = "100")]
        max_attempts: Option<usize>,
    },
    /// MCP server
    Mcp {
        #[arg(long, short, default_value = "stdio")]
        transport: TransportType,
        #[arg(long, short, default_value = "2493")]
        /// Port to listen on for HTTP transport (required if --transport is set to http)
        port: Option<u16>,
        #[arg(long, default_value = "localhost")]
        /// Host to bind for HTTP transport (required if --transport is set to http)
        host: Option<String>,
    },
}
