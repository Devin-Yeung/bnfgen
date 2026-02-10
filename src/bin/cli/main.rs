mod cli;

use bnfgen::grammar::raw::RawGrammar;
use bnfgen::report::{Reporter, Style};
use clap::Parser;
use miette::Report;
use std::sync::Arc;

use crate::cli::Cli;

fn main() {
    let args = Cli::parse();

    let text = std::fs::read_to_string(&args.grammar).unwrap();
    let text = Arc::new(text);
    let mut reporter = Reporter::new(Style::NoColor);

    let report_and = |reporter: &mut Reporter, e, v| {
        let diagnostic = Report::from(e).with_source_code(text.clone());
        reporter.push(diagnostic);
        v
    };

    let shutdown = |reporter: &Reporter| -> ! {
        let msg = reporter.report_to_string();
        if !msg.is_empty() {
            eprintln!("{}", msg);
            std::process::exit(1);
        }
        std::process::exit(0);
    };

    let grammar = match RawGrammar::parse(text.as_str()) {
        Ok(g) => g,
        Err(e) => {
            reporter.push(e);
            shutdown(&reporter);
        }
    };

    #[rustfmt::skip]
    let continue_check = grammar.check_repeats()
        .map_or_else(|e| report_and(&mut reporter, e, false), |_| true) &&
        grammar.check_duplicate()
        .map_or_else(|e| report_and(&mut reporter, e, false), |_| true) &&
        grammar.check_repeats()
        .map_or_else(|e| report_and(&mut reporter, e, false), |_| true) ;

    if continue_check {
        let graph = grammar.graph();
        let _ = graph
            .check_trap_loop()
            .map_err(|e| report_and(&mut reporter, e, false));
        if let Some(start) = &args.check_unused {
            let _ = graph
                .check_unused(start)
                .map_err(|e| report_and(&mut reporter, e, false));
        }
    }

    shutdown(&reporter);
}
