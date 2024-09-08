use crate::error::Error;
use crate::span::Span;
use crate::token::{LexicalError, Token};

pub(crate) fn convert_parse_error(
    e: lalrpop_util::ParseError<usize, Token, LexicalError>,
) -> Error {
    match e {
        lalrpop_util::ParseError::UnrecognizedToken {
            token: (l, _, r),
            expected,
        } => {
            let expected = expected.join(", ");
            Error::UnrecognizedToken {
                span: Span::new(l, r),
                expect: expected,
            }
        }
        lalrpop_util::ParseError::ExtraToken { token: (l, _, r) } => Error::ExtraToken {
            span: Span::new(l, r),
        },
        lalrpop_util::ParseError::InvalidToken { .. } => unreachable!("Should raised by logos"),
        lalrpop_util::ParseError::UnrecognizedEof { location, expected } => {
            let expected = expected.join(", ");
            Error::UnrecognizedEof {
                span: Span::new(location - 1, location),
                expect: expected,
            }
        }
        lalrpop_util::ParseError::User { error } => Error::LexicalError(error),
    }
}
