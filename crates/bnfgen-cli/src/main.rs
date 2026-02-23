mod app;
mod cli;
mod mcp;

use crate::app::App;
use crate::cli::{Cli, Command};
use crate::mcp::{BnfgenMCP, BnfgenSettings};
use anyhow::Result;
use clap::Parser;
use rmcp::transport::stdio;
use rmcp::ServiceExt;

#[tokio::main]
async fn main() -> Result<()> {
    let args = Cli::parse();

    match args.command {
        Command::Check {
            grammar,
            start,
            strict,
        } => {
            let grammar = std::fs::read_to_string(grammar)?;
            let app = App::new(grammar);
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
                return Err(app.diagnostics());
            }

            Ok(())
        }

        Command::Gen {
            grammar,
            start,
            count,
            seed,
            max_steps,
            max_attempts,
        } => {
            let grammar = std::fs::read_to_string(grammar)?;
            let app = App::new(grammar);

            // perform parsing and linting
            let raw = app.parse()?;
            let checked = app.lint(raw)?;

            // generate output
            let outputs = app.generate(checked, start, count, seed, max_steps, max_attempts)?;
            for output in outputs {
                println!("{}", output);
            }
            Ok(())
        }

        Command::Mcp {} => {
            let service = BnfgenMCP::new(BnfgenSettings::builder().build())
                .serve(stdio())
                .await?;
            service.waiting().await?;
            Ok(())
        }
    }
}
