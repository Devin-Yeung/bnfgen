//! Error reporting with rich diagnostics.
//!
//! This module provides [`Reporter`] for collecting and rendering diagnostic
//! messages using the [miette] library. Reporters can display errors with
//! source code annotations, labels, and helpful context.
//!
//! [miette]: https://docs.rs/miette

use miette::{GraphicalReportHandler, GraphicalTheme, Report};

/// Output style for error reports.
#[derive(Debug)]
pub enum Style {
    /// Fancy colored output (currently unimplemented).
    Fancy,
    /// Plain Unicode output without colors.
    NoColor,
}

/// Collector and renderer for diagnostic messages.
///
/// `Reporter` accumulates diagnostics and renders them with rich formatting
/// including source code annotations, labels, and context. Uses the [miette]
/// library for graphical error reports.
///
/// # Example
///
/// ```rust
/// use bnfgen::{RawGrammar, Reporter, Style};
///
/// let grammar = RawGrammar::parse("<E> ::= <Undefined>;").unwrap();
/// match grammar.to_checked() {
///     Ok(_) => println!("Valid"),
///     Err(e) => {
///         let mut reporter = Reporter::new(Style::NoColor);
///         reporter.push(e);
///         eprintln!("{}", reporter.report_to_string());
///     }
/// }
/// ```
///
/// [miette]: https://docs.rs/miette
pub struct Reporter {
    handler: GraphicalReportHandler,
    diagnostics: Vec<Report>,
}

impl Reporter {
    /// Creates a new reporter with the specified output style.
    ///
    /// # Arguments
    ///
    /// * `style` - The output style (fancy or no-color)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::report::{Reporter, Style};
    ///
    /// let reporter = Reporter::new(Style::NoColor);
    /// ```
    pub fn new(style: Style) -> Self {
        let theme = match style {
            Style::Fancy => todo!(),
            Style::NoColor => GraphicalTheme::unicode_nocolor(),
        };

        Self {
            handler: GraphicalReportHandler::new_themed(theme),
            diagnostics: Vec::new(),
        }
    }

    /// Adds a diagnostic to the reporter.
    ///
    /// # Arguments
    ///
    /// * `diagnostic` - Any type that can be converted to a `miette::Report`
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::report::{Reporter, Style};
    /// use bnfgen::Error;
    ///
    /// let mut reporter = Reporter::new(Style::NoColor);
    /// // reporter.push(error);
    /// ```
    pub fn push<T: Into<Report>>(&mut self, diagnostic: T) {
        self.diagnostics.push(diagnostic.into());
    }

    /// Adds multiple diagnostics to the reporter.
    ///
    /// # Arguments
    ///
    /// * `diagnostic` - An iterator of items that can be converted to `miette::Report`
    pub fn extend<T, I>(&mut self, diagnostic: I)
    where
        T: Into<Report>,
        I: IntoIterator<Item = T>,
    {
        self.diagnostics
            .extend(diagnostic.into_iter().map(Into::into));
    }

    /// Renders all diagnostics to a writer.
    ///
    /// # Arguments
    ///
    /// * `writer` - Any type implementing `std::fmt::Write`
    ///
    /// # Errors
    ///
    /// Returns a `std::fmt::Error` if writing fails.
    pub fn report<W: std::fmt::Write>(&self, writer: &mut W) -> std::fmt::Result {
        for diagnostic in &self.diagnostics {
            self.handler.render_report(writer, diagnostic.as_ref())?
        }
        Ok(())
    }

    /// Renders all diagnostics to a string.
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::report::{Reporter, Style};
    ///
    /// let reporter = Reporter::new(Style::NoColor);
    /// // let output = reporter.report_to_string();
    /// ```
    pub fn report_to_string(&self) -> String {
        let mut buffer = String::new();
        self.report(&mut buffer).unwrap();
        buffer
    }

    /// Returns `true` if there are any diagnostics in the reporter.
    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}
