mod app;
mod cli;
mod mcp;

use crate::app::App;
use crate::cli::{Cli, Command};
use anyhow::Result;
use bnfgen::generator::GeneratorSettings;
use bnfgen::{Error, Generator};
use clap::Parser;
use rand::rngs::StdRng;
use rand::SeedableRng;

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
                return Err(app.diagnostics());
            }

            Ok(())
        }

        Command::Gen {
            grammar,
            start,
            count,
            seed,
            max_attempts,
        } => {
            let grammar = std::fs::read_to_string(grammar)?;

            let app = App::new(grammar)?;

            let raw = app.parse()?;
            let checked = app.lint(raw)?;

            let settings = GeneratorSettings::builder()
                .max_depth(Some(max_attempts))
                .build();
            let generator = Generator::builder()
                .grammar(checked)
                .settings(settings)
                .build();

            let mut outputs = Vec::with_capacity(count);

            let mut rng = match seed {
                Some(s) => StdRng::seed_from_u64(s),
                None => StdRng::from_rng(&mut rand::rng()),
            };

            for _ in 0..count {
                loop {
                    match generator.generate(start.clone(), &mut rng) {
                        Ok(line) => {
                            outputs.push(line);
                            break;
                        }
                        Err(Error::MaxDepthExceeded) => {
                            continue;
                        }
                        Err(err) => return Err(err.into()),
                    }
                }
            }

            for output in outputs {
                println!("{}", output);
            }

            Ok(())
        }
    }
}
