use logos::{Logos, SpannedIter};

use crate::error::Error;
use crate::token::{LexicalError, Token};

pub type Spanned<Tok, Loc, Error> = Result<(Loc, Tok, Loc), Error>;

pub struct Lexer<'input> {
    // instead of an iterator over characters, we have a token iterator
    token_stream: SpannedIter<'input, Token>,
}

impl<'input> Lexer<'input> {
    pub fn new(input: &'input str) -> Self {
        Self {
            token_stream: Token::lexer(input).spanned(),
        }
    }
}

impl<'input> Iterator for Lexer<'input> {
    type Item = Spanned<Token, usize, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.token_stream.next().map(|(token, span)| match token {
            Ok(tok) => Ok((span.start, tok, span.end)),
            Err(e) => match e {
                LexicalError::InternalInvalidToken => {
                    let err = LexicalError::InvalidToken(span.into());
                    Err(Error::LexicalError(err))
                }
                e => Err(Error::LexicalError(e)),
            },
        })
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn it_works() {
        let input = include_str!("../examples/bnf.bnfgen");
        let lexer = super::Lexer::new(input);
        let tokens: Vec<_> = lexer.collect();
        insta::assert_debug_snapshot!(tokens);
    }
}
