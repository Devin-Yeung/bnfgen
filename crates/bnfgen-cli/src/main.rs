mod cli;
mod mcp;

use bnfgen::generator::Generator;
use bnfgen::grammar::checked::CheckedGrammar;
use bnfgen::grammar::raw::RawGrammar;
use bnfgen::report::{Reporter, Style};
use clap::Parser;
use miette::Report;
use rand::SeedableRng;
use std::sync::Arc;

use crate::cli::{Cli, Command};

struct Context {
    text: Arc<String>,
    reporter: Reporter,
}

impl Context {
    fn new(text: String) -> Self {
        Self {
            text: Arc::new(text),
            reporter: Reporter::new(Style::NoColor),
        }
    }

    fn report_error(&mut self, e: impl Into<Report>) {
        let diagnostic = e.into().with_source_code(self.text.clone());
        self.reporter.push(diagnostic);
    }

    fn has_errors(&self) -> bool {
        !self.reporter.report_to_string().is_empty()
    }

    fn shutdown(&self) -> ! {
        let msg = self.reporter.report_to_string();
        if !msg.is_empty() {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
        std::process::exit(0);
    }
}

fn main() {
    let args = Cli::parse();

    match args.command {
        Command::Check {
            grammar,
            check_unused,
        } => check_grammar(grammar, check_unused),
        Command::Gen {
            grammar,
            start,
            count,
            seed,
        } => generate_strings(grammar, start, count, seed),
    }
}

fn parse_and_validate(ctx: &mut Context) -> Option<RawGrammar> {
    // We need to clone the Arc to avoid borrowing issues
    let text = Arc::clone(&ctx.text);
    let grammar = match RawGrammar::parse(&**text) {
        Ok(g) => g,
        Err(e) => {
            ctx.report_error(e);
            ctx.shutdown();
        }
    };

    let checks = [
        grammar.check_repeats(),
        grammar.check_duplicate(),
        grammar.check_undefined(),
    ];

    let all_passed = checks.iter().all(|result| {
        result.as_ref().map_or_else(
            |e| {
                ctx.report_error(e.clone());
                false
            },
            |_| true,
        )
    });

    if all_passed {
        let graph = grammar.graph();
        let _ = graph.check_trap_loop().map_err(|e| {
            ctx.report_error(e);
            false
        });
    }

    if ctx.has_errors() {
        ctx.shutdown();
    }

    Some(grammar)
}

fn check_grammar(grammar_path: std::path::PathBuf, check_unused: Option<String>) {
    let text = std::fs::read_to_string(&grammar_path).unwrap();
    let mut ctx = Context::new(text);

    let grammar = parse_and_validate(&mut ctx).unwrap();

    if let Some(start) = &check_unused {
        let graph = grammar.graph();
        let _ = graph.check_unused(start).map_err(|e| {
            ctx.report_error(e);
        });
    }

    ctx.shutdown();
}

fn generate_strings(
    grammar_path: std::path::PathBuf,
    start: String,
    count: usize,
    seed: Option<u64>,
) {
    let text = std::fs::read_to_string(&grammar_path).unwrap();
    let mut ctx = Context::new(text);

    let grammar = parse_and_validate(&mut ctx).unwrap();

    let checked = match grammar.to_checked() {
        Ok(g) => g,
        Err(e) => {
            ctx.report_error(e);
            ctx.shutdown();
        }
    };

    generate(checked, &start, count, seed);
}

fn generate(grammar: CheckedGrammar, start: &str, count: usize, seed: Option<u64>) {
    let generator = Generator::new(grammar);

    if let Some(s) = seed {
        let mut rng = rand::rngs::StdRng::seed_from_u64(s);
        for _ in 0..count {
            match generator.generate(start, &mut rng) {
                Ok(output) => println!("{}", output),
                Err(e) => {
                    eprintln!("Error during generation: {}", e);
                    std::process::exit(1);
                }
            }
        }
    } else {
        let mut rng = rand::rng();
        for _ in 0..count {
            match generator.generate(start, &mut rng) {
                Ok(output) => println!("{}", output),
                Err(e) => {
                    eprintln!("Error during generation: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}
