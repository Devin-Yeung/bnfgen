use crate::span::Span;

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
    #[error("Undefined non-terminal")]
    UndefinedNonTerminal {
        #[label("this non-terminal is undefined")]
        span: Span,
    },
    #[error("Duplicated rules found")]
    DuplicatedRules {
        #[label("this rule is duplicated")]
        span: Span,
        #[label("previous defined here")]
        prev: Span,
    },
    #[error("Invalid repeat range")]
    InvalidRepeatRange {
        #[label("min should be less than or equal to max")]
        span: Span,
    },
    #[error("Found unreachable rules")]
    UnreachableRules {
        #[label(collection, "this rule is unreachable")]
        spans: Vec<Span>,
    },
    #[error("May be trapped in a dead loop")]
    TrapLoop {
        #[label(collection, "this rule may be trapped in a dead loop")]
        spans: Vec<Span>,
    },
    #[error(transparent)]
    #[diagnostic(transparent)]
    LexicalError(#[from] crate::token::LexicalError),
}
