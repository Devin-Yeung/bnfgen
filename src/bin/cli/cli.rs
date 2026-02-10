use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug, Clone)]
pub struct Cli {
    #[arg(short, long)]
    /// Path to the BNF grammar file
    pub grammar: PathBuf,
    #[arg(long)]
    /// Check for unreachable rules (need to give the starting rule)
    pub check_unused: Option<String>,
}
