use std::fmt::Display;
use std::{fmt::Formatter, num::ParseIntError};

use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+", skip r"#.*\n?", error = LexicalError)]
pub enum Token {
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("|")]
    OR,
    #[token(",")]
    COMMA,
    #[token("::=")]
    Def,
    #[token(";")]
    Semi,
    #[regex("[0-9]+", |lex| lex.slice().parse::<u32>())]
    Int(u32),
    #[regex("<(?:[a-zA-Z]*)>", |lex| lex.slice()[1..lex.slice().len() - 1].to_string())]
    NonTerminal(String),
    #[regex(r#"'(?:\\'|[^(')])*'|"(?:\\"|[^(")])*""#, |lex| lex.slice()[1..lex.slice().len() - 1].to_string())]
    Terminal(String),
}

#[derive(Default, Debug, Clone, PartialEq)]
pub enum LexicalError {
    InvalidInteger(ParseIntError),
    #[default]
    InvalidToken,
}

impl From<ParseIntError> for LexicalError {
    fn from(err: ParseIntError) -> Self {
        LexicalError::InvalidInteger(err)
    }
}

impl Display for LexicalError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            LexicalError::InvalidInteger(err) => write!(f, "invalid integer: {}", err),
            LexicalError::InvalidToken => write!(f, "invalid token"),
        }
    }
}
