use crate::regex::Regex;
use crate::span::Span;
use std::rc::Rc;

#[derive(Debug, Clone)]
pub(crate) enum SymbolKind {
    Terminal(Rc<String>),
    NonTerminal(Rc<String>),
    Regex(Rc<Regex>),
}

impl SymbolKind {
    pub fn non_re_terminal(&self) -> Option<&str> {
        match self {
            SymbolKind::Terminal(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        match self {
            SymbolKind::Terminal(_) | SymbolKind::Regex(_) => true,
            _ => false,
        }
    }

    // get the non-terminal symbol if it is a non-terminal symbol, else none
    pub fn non_terminal(&self) -> Option<&str> {
        match self {
            SymbolKind::NonTerminal(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub struct Symbol {
    pub(crate) kind: SymbolKind,
    pub(crate) span: Span,
}

impl Symbol {
    pub fn is_terminal(&self) -> bool {
        self.kind.is_terminal()
    }

    pub fn non_terminal(&self) -> Option<&str> {
        self.kind.non_terminal()
    }
}
