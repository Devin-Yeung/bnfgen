use std::sync::Arc;

use bnfgen::error::Result;
use bnfgen::generator::Generator;
use bnfgen::grammar::raw::RawGrammar;
use bnfgen::report::Style;
use miette::Report;

fn main() {
    let mut report = bnfgen::report::Reporter::new(Style::NoColor);
    let input = include_str!("set-algebra.bnfgen");
    let source = Arc::new(input.to_string());
    let _ = run(input).map_err(|e| {
        let diagnostic = Report::from(e).with_source_code(source);
        report.push(diagnostic);
        print!("{}", report.report_to_string());
    });
}

fn run(input: &str) -> Result<()> {
    let grammar = RawGrammar::parse(input)?.to_checked()?;
    let gen = Generator::builder().grammar(grammar).build();
    let out = gen.generate("Program", &mut rand::rng());
    println!("{}", out);
    Ok(())
}
