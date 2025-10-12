use std::num::ParseIntError;

use crate::span::Span;
use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(skip r"[ \t\n\f]+", skip r"//.*\n?", error = LexicalError)]
pub enum Token {
    #[token("{")]
    LBrace,
    #[token("}")]
    RBrace,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
    #[token("|")]
    Or,
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("::=")]
    Def,
    #[token("<")]
    LAngle,
    #[token(">")]
    RAngle,
    #[token(";")]
    Semi,
    #[token("re")]
    Re,
    #[token("decl")]
    Decl,
    #[token("decl_defer")]
    DeclDefer,
    #[token("ref")]
    Ref,
    #[rustfmt::skip]
    #[regex("[0-9]|[1-9][0-9]*", |lex| {
        match lex.slice().parse::<usize>() {
            Ok(t) => Ok(t),
            Err(e) => Err(LexicalError::InvalidInteger(e, lex.span().into()))
        }
    })]
    Int(usize),
    #[regex("[a-zA-Z-_0-9]*", |lex| lex.slice().to_string())]
    Id(String),
    #[rustfmt::skip]
    #[regex(r#""(\\["nrt\\]|[^"\\])*""#, |lex| {
        let text = &lex.slice()[1..lex.slice().len() - 1];
        text.replace("\\\"", "\"")
            .replace("\\n", "\n")
            .replace("\\t", "\t")
            .replace("\\r", "\r")
            .to_string()
    })]
    Str(String),
}

#[derive(thiserror::Error, miette::Diagnostic, Default, Debug, Clone, PartialEq, Eq)]
pub enum LexicalError {
    #[error("Invalid integer")]
    InvalidInteger(ParseIntError, #[label("this int is invalid")] Span),
    #[error("Invalid token")]
    InvalidToken(#[label("this token is invalid")] Span),
    // see: https://github.com/maciejhirsz/logos/issues/352
    #[default]
    #[error("Internal Error. Please file an issue if you see this")]
    InternalInvalidToken,
}
