pub mod error;
pub mod generator;
pub mod grammar;
mod lexer;
pub mod parse_tree;
mod regex;
pub mod report;
mod span;
mod token;
mod utils;

use lalrpop_util::lalrpop_mod;
lalrpop_mod!(parser);
