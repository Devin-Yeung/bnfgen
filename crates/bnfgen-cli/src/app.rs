use anyhow::Result;
use bnfgen::generator::GeneratorSettings;
use bnfgen::grammar::raw::RawGrammar;
use bnfgen::report::{Reporter, Style};
use bnfgen::{CheckedGrammar, Error};
use miette::Report;
use rand::SeedableRng;
use std::cell::RefCell;
use std::sync::Arc;

pub struct App {
    grammar: Arc<String>,
    reporter: RefCell<Reporter>,
}

impl App {
    pub fn new<T: Into<String>>(grammar: T) -> Self {
        let reporter = Reporter::new(Style::NoColor);

        Self {
            grammar: Arc::new(grammar.into()),
            reporter: RefCell::new(reporter),
        }
    }

    pub fn parse(&self) -> Result<RawGrammar> {
        RawGrammar::parse(self.grammar.as_str()).map_err(|e| self.fail_fast(e))
    }

    /// performs additional checks that are not strictly necessary for correctness,
    /// but can help catch common mistakes and improve the quality of the grammar
    /// return true if the grammar passes all checks, false otherwise
    pub fn strict_lint(&self, grammar: &RawGrammar, start: String) -> bool {
        let mut has_errors = false;
        let graph = grammar.graph();

        if let Err(e) = graph.check_unused(start) {
            self.report(e);
            has_errors = true;
        }

        if let Err(e) = graph.check_trap_loop() {
            self.report(e);
            has_errors = true;
        }

        !has_errors
    }

    pub fn lint(&self, grammar: RawGrammar) -> Result<CheckedGrammar> {
        let correctness = [
            grammar.check_undefined(),
            grammar.check_duplicate(),
            grammar.check_repeats(),
        ];

        // these checks are independent, so we can run them in parallel and collect all errors
        for check in correctness {
            if let Err(e) = check {
                self.report(e);
            }
        }

        match grammar.to_checked() {
            Ok(g) => Ok(g),
            Err(_) => {
                // we have already reported all the errors, so we can just return a generic error here
                Err(self.diagnostics())
            }
        }
    }

    pub fn generate(
        &self,
        grammar: CheckedGrammar,
        start: String,
        count: usize,
        seed: Option<u64>,
        max_steps: Option<usize>,
    ) -> Result<Vec<String>> {
        let settings = GeneratorSettings::builder().max_steps(max_steps).build();

        let generator = bnfgen::Generator::builder()
            .grammar(grammar)
            .settings(settings)
            .build();

        let mut outputs = Vec::with_capacity(count);

        let mut rng = match seed {
            Some(s) => rand::rngs::StdRng::seed_from_u64(s),
            None => rand::rngs::StdRng::from_rng(&mut rand::rng()),
        };

        for _ in 0..count {
            match generator.generate(&start, &mut rng) {
                Ok(output) => outputs.push(output),
                Err(e) => match e {
                    Error::MaxDepthExceeded => continue,
                    e => return Err(self.fail_fast(e)),
                },
            }
        }

        Ok(outputs)
    }

    /// early exit with an error, reporting it to the user
    pub fn fail_fast(&self, e: impl Into<Report>) -> anyhow::Error {
        self.report(e);
        self.diagnostics()
    }

    pub fn report(&self, e: impl Into<Report>) {
        let mut reporter = self.reporter.borrow_mut();
        let diagnostic = e.into().with_source_code(self.grammar.clone());
        reporter.push(diagnostic);
    }

    pub fn diagnostics(&self) -> anyhow::Error {
        let reporter = self.reporter.borrow();
        let diagnostics = reporter.report_to_string();
        anyhow::anyhow!(diagnostics)
    }
}
