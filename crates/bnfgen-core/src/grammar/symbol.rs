//! Symbol types for BNF grammar.
//!
//! This module defines the types representing symbols in a BNF grammar:
//!
//! - [`Terminal`] - A literal string value (e.g., `"hello"`)
//! - [`NonTerminal`] - A reference to another rule (e.g., `<S>`)
//! - [`Ty`] - Type annotation for typed non-terminals (e.g., `"int"`)
//! - [`SymbolKind`] - Enum of all possible symbol kinds

use crate::regex::Regex;
use crate::span::Span;
use std::hash::Hash;
use std::rc::Rc;

/// A terminal symbol - a literal string value.
///
/// Terminals are the actual strings that appear in the generated output.
/// They are stored as `Rc<String>` for efficient cloning during generation.
///
/// # Example
///
/// ```text
/// "hello"  // This is a terminal
/// ```
pub type Terminal = Rc<String>;

/// A non-terminal symbol referencing another grammar rule.
///
/// Non-terminals can be untyped (e.g., `<S>`) or typed (e.g., `<E: "int">`).
/// Typed non-terminals enable polymorphism in grammars.
///
/// # Example
///
/// ```rust
/// use bnfgen::grammar::symbol::{NonTerminal, Ty};
///
/// // Create an untyped non-terminal
/// let s = NonTerminal::untyped("S");
///
/// // Create a typed non-terminal
/// let int_expr = NonTerminal::typed("E", Ty::typed("int"));
/// ```
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
    /// Creates an untyped non-terminal.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the non-terminal (e.g., "S")
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::grammar::symbol::NonTerminal;
    ///
    /// let s = NonTerminal::untyped("S");
    /// ```
    pub fn untyped<S: Into<String>>(name: S) -> Self {
        NonTerminal {
            name: Rc::new(name.into()),
            ty: Ty::Untyped,
        }
    }

    /// Creates a typed non-terminal.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the non-terminal (e.g., "E")
    /// * `ty` - The type annotation (e.g., `Ty::typed("int")`)
    ///
    /// # Example
    ///
    /// ```rust
    /// use bnfgen::grammar::symbol::{NonTerminal, Ty};
    ///
    /// let e = NonTerminal::typed("E", Ty::typed("int"));
    /// ```
    pub fn typed<S: Into<String>>(name: S, ty: Ty) -> Self {
        NonTerminal {
            name: Rc::new(name.into()),
            ty,
        }
    }

    /// Returns the name of this non-terminal as a string slice.
    pub fn as_str(&self) -> &str {
        self.name.as_str()
    }
}

/// Type annotation for typed non-terminals.
///
/// Types enable polymorphic grammar rules. For example:
///
/// ```text
/// <E: "int">  ::= "1" | "2" | <E: "int"> "+" <E: "int"> ;
/// <E: "bool"> ::= "true" | "false" | <E: "bool"> "&" <E: "bool"> ;
/// ```
///
/// # Example
///
/// ```rust
/// use bnfgen::grammar::symbol::Ty;
///
/// // Create types from strings
/// let int_ty = Ty::typed("int");
/// let bool_ty = Ty::typed("bool");
/// let untyped = Ty::untyped();
/// ```
#[derive(Debug, Eq, PartialEq)]
pub enum Ty {
    /// Untyped non-terminal (e.g., `<S>`)
    Untyped,
    /// Typed non-terminal (e.g., `<E: "int">`)
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
    /// Creates an untyped `Ty`.
    pub fn untyped() -> Self {
        Ty::Untyped
    }

    /// Creates a typed `Ty` from a string.
    ///
    /// # Arguments
    ///
    /// * `s` - The type name (e.g., "int", "bool")
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
    /// Returns the type name if this is a typed `Ty`, otherwise `None`.
    pub fn ty(&self) -> Option<&str> {
        match self {
            Ty::Untyped => None,
            Ty::Typed(s) => Some(s.as_str()),
        }
    }
}

/// The kind of a symbol - terminal, non-terminal, or regex.
///
/// `SymbolKind` represents the three types of symbols that can appear
/// in a BNF grammar:
///
/// - **Terminals**: Literal string values (e.g., `"hello"`)
/// - **Non-terminals**: References to other rules (e.g., `<S>`)
/// - **Regex**: Pattern-based generators (e.g., `re("[a-zA-Z]+")`)
///
/// # Example
///
/// The `SymbolKind` enum is typically created internally during parsing,
/// but you can construct it directly:
///
/// ```rust
/// use bnfgen::grammar::symbol::{SymbolKind, NonTerminal};
/// use std::rc::Rc;
///
/// let terminal = SymbolKind::Terminal(Rc::new("hello".to_string()));
/// let non_terminal = SymbolKind::NonTerminal(
///     NonTerminal::untyped("S")
/// );
/// ```
#[derive(Debug, Clone)]
pub enum SymbolKind {
    /// A literal terminal string
    Terminal(Terminal),
    /// A reference to another rule
    NonTerminal(NonTerminal),
    /// A regex pattern for generating strings
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
    /// Returns the terminal string if this is a non-regex terminal.
    pub fn non_re_terminal(&self) -> Option<&str> {
        match self {
            SymbolKind::Terminal(s) => Some(s.as_str()),
            _ => None,
        }
    }

    /// Returns `true` if this symbol is a terminal (including regex).
    ///
    /// Terminals are symbols that can be directly output without further
    /// expansion. Both literal terminals and regex-generated terminals
    /// return `true`.
    pub fn is_terminal(&self) -> bool {
        matches!(self, SymbolKind::Terminal(_) | SymbolKind::Regex(_))
    }

    /// Returns the non-terminal name if this is a non-terminal, otherwise `None`.
    pub fn non_terminal(&self) -> Option<&str> {
        match self {
            SymbolKind::NonTerminal(s) => Some(s.as_str()),
            _ => None,
        }
    }
}

/// A symbol with an associated source span.
///
/// `Symbol` is the internal representation used during parsing,
/// containing both the `SymbolKind` and its location in the source text
/// for error reporting.
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
    /// Returns `true` if this symbol is a terminal (including regex).
    pub fn is_terminal(&self) -> bool {
        self.kind.is_terminal()
    }

    /// Returns the non-terminal name if this is a non-terminal, otherwise `None`.
    pub fn non_terminal(&self) -> Option<&str> {
        self.kind.non_terminal()
    }
}
