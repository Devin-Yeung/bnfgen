use miette::{GraphicalReportHandler, GraphicalTheme, Report};

#[derive(Debug)]
pub enum Style {
    Fancy,
    NoColor,
}

pub struct Reporter {
    handler: GraphicalReportHandler,
    diagnostics: Vec<Report>,
}

impl Reporter {
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

    pub fn push<T: Into<Report>>(&mut self, diagnostic: T) {
        self.diagnostics.push(diagnostic.into());
    }

    pub fn extend<T, I>(&mut self, diagnostic: I)
    where
        T: Into<Report>,
        I: IntoIterator<Item = T>,
    {
        self.diagnostics
            .extend(diagnostic.into_iter().map(Into::into));
    }

    pub fn report<W: std::fmt::Write>(&self, writer: &mut W) -> std::fmt::Result {
        for diagnostic in &self.diagnostics {
            self.handler.render_report(writer, diagnostic.as_ref())?
        }
        Ok(())
    }

    pub fn report_to_string(&self) -> String {
        let mut buffer = String::new();
        self.report(&mut buffer).unwrap();
        buffer
    }

    pub fn has_diagnostics(&self) -> bool {
        !self.diagnostics.is_empty()
    }
}
