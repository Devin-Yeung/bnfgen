mod app;
mod cli;
mod mcp;

use crate::app::App;
use crate::cli::{Cli, Command};
use anyhow::Result;
use clap::Parser;

fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Check {
            grammar,
            start,
            strict,
        } => {
            let grammar = std::fs::read_to_string(grammar)?;
            let app = App::new(grammar)?;
            let raw = app.parse()?;

            let mut pass = true;

            if strict {
                pass = app.strict_lint(
                    &raw,
                    start.expect("starting non-terminal is required when --strict is set"),
                );
            }

            let _checked = app.lint(raw)?;

            if !pass {
                return Err(app.diagnotics());
            }

            Ok(())
        }

        Command::Gen {
            grammar,
            start,
            count,
            seed,
        } => {
            todo!()
        }
    }
}
