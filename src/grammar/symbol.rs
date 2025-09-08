use crate::regex::Regex;
use crate::span::Span;
use std::hash::Hash;
use std::rc::Rc;

pub type Terminal = Rc<String>;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NonTerminal {
    pub(crate) name: Rc<String>,
    pub(crate) ty: Ty,
}

impl Hash for NonTerminal {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.name.hash(state);
        self.ty.hash(state);
    }
}

impl NonTerminal {
    pub fn untyped<S: Into<String>>(name: S) -> Self {
        NonTerminal {
            name: Rc::new(name.into()),
            ty: Ty::Untyped,
        }
    }

    pub fn typed<S: Into<String>>(name: S, ty: Ty) -> Self {
        NonTerminal {
            name: Rc::new(name.into()),
            ty,
        }
    }

    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

#[derive(Debug, Eq, PartialEq)]
pub enum Ty {
    Untyped,
    Typed(Rc<String>),
}

impl From<String> for Ty {
    fn from(s: String) -> Self {
        Ty::Typed(Rc::new(s))
    }
}

impl Hash for Ty {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Ty::Untyped => { /* do nothing */ }
            Ty::Typed(s) => s.hash(state),
        }
    }
}

impl Ty {
    pub fn untyped() -> Self {
        Ty::Untyped
    }

    pub fn typed<S: Into<String>>(s: S) -> Self {
        Ty::Typed(Rc::new(s.into()))
    }
}

impl Clone for Ty {
    fn clone(&self) -> Self {
        match self {
            Ty::Untyped => Ty::Untyped,
            Ty::Typed(s) => Ty::Typed(s.clone()),
        }
    }
}

impl Ty {
    pub fn ty(&self) -> Option<&str> {
        match self {
            Ty::Untyped => None,
            Ty::Typed(s) => Some(s.as_str()),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SymbolKind {
    Terminal(Terminal),
    NonTerminal(NonTerminal),
    Regex(Rc<Regex>),
}

impl Hash for SymbolKind {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            SymbolKind::Terminal(s) => s.hash(state),
            SymbolKind::NonTerminal(s) => s.hash(state),
            SymbolKind::Regex(s) => s.hash(state),
        }
    }
}

impl SymbolKind {
    pub fn non_re_terminal(&self) -> Option<&str> {
        match self {
            SymbolKind::Terminal(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn is_terminal(&self) -> bool {
        matches!(self, SymbolKind::Terminal(_) | SymbolKind::Regex(_))
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

impl Hash for Symbol {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.kind.hash(state);
    }
}

impl Symbol {
    pub fn is_terminal(&self) -> bool {
        self.kind.is_terminal()
    }

    pub fn non_terminal(&self) -> Option<&str> {
        self.kind.non_terminal()
    }
}
