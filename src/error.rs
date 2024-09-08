use crate::span::Span;
use miette;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(thiserror::Error, miette::Diagnostic, Debug, Eq, PartialEq, Clone)]
pub enum Error {
    #[error("Unrecognized token")]
    UnrecognizedToken {
        #[label("expect {expect}")]
        span: Span,
        expect: String,
    },
    #[error("Unexpected extra token")]
    ExtraToken {
        #[label("this extra token is unexpected")]
        span: Span,
    },
    #[error("Unrecognized EOF")]
    UnrecognizedEof {
        #[label("expect {expect}")]
        span: Span,
        expect: String,
    },
    #[error(transparent)]
    #[diagnostic(transparent)]
    LexicalError(#[from] crate::token::LexicalError),
}
