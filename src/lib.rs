pub mod generator;
pub mod grammar;
mod lexer;
mod regex;
mod token;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(parser);
