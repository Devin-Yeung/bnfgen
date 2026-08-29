//! Normalized public-seam snapshot of a [`ParsedDocument`].
//!
//! One entry point: verbatim source, then an eza-style Unicode tree of
//! tokens, diagnostics, recovery, and rules. Trivia stays a token leaf.
//! Required children that are absent render as `{role} missing`; optional
//! absences are omitted. Every view that exposes [`StructuralState`] shows
//! it on that node's label, before the raw lexeme. String literals use
//! `unterminated` instead of `partial` because that is the same fact and
//! the more specific word. Newlines are stripped from inline lexemes so the
//! tree stays one line per leaf; the source dump still shows them.

use std::fmt::Debug;
use std::ops::Range;

use bnfgen_syntax::{
    AlternativeSyntax, IntegerSyntax, NonTerminalSyntax, ParsedDocument, RecoveryObservation,
    RegexSyntax, RepeatSyntax, RuleSyntax, StringSyntax, StructuralState, SymbolKind, SymbolSyntax,
    SyntaxDiagnostic, SyntaxToken,
};
use termtree::Tree;

/// Full reviewable dump of a document: source plus public-view tree.
pub fn snapshot(doc: &ParsedDocument) -> String {
    format!(
        "<input>\n{}</input>\n{}",
        doc.source(),
        document(doc).into_tree()
    )
}

struct Label(Vec<String>);

impl Label {
    fn new() -> Self {
        Self(Vec::new())
    }

    fn range(self, range: Range<usize>) -> Self {
        self.word(format!("{}..{}", range.start, range.end))
    }

    fn word(mut self, part: impl Into<String>) -> Self {
        let part = part.into();
        if !part.is_empty() {
            self.0.push(part);
        }
        self
    }

    fn debug(self, value: impl Debug) -> Self {
        self.word(format!("{value:?}"))
    }

    fn lexeme(self, text: &str) -> Self {
        let stripped: String = text.chars().filter(|c| !matches!(c, '\n' | '\r')).collect();
        self.word(stripped)
    }

    fn state(self, state: StructuralState) -> Self {
        self.word(match state {
            StructuralState::Complete => "complete",
            StructuralState::Partial => "partial",
        })
    }

    fn flag(self, word: &'static str, on: bool) -> Self {
        if on {
            self.word(word)
        } else {
            self
        }
    }

    fn finish(self) -> String {
        self.0.join(" ")
    }
}

struct Node {
    label: String,
    children: Vec<Node>,
}

impl Node {
    fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            children: Vec::new(),
        }
    }

    fn branch(label: impl Into<String>, children: impl IntoIterator<Item = Node>) -> Self {
        Self {
            label: label.into(),
            children: children.into_iter().collect(),
        }
    }

    fn push(&mut self, child: Node) {
        self.children.push(child);
    }

    fn extend(&mut self, children: impl IntoIterator<Item = Node>) {
        self.children.extend(children);
    }

    /// Always emit a child. `None` becomes `{role} missing`.
    fn required(&mut self, role: &'static str, child: Option<Node>) {
        self.push(child.unwrap_or_else(|| Node::new(format!("{role} missing"))));
    }

    fn required_range(&mut self, role: &'static str, range: Option<Range<usize>>) {
        self.required(role, range.map(|range| range_node(role, range)));
    }

    fn present(&mut self, child: Option<Node>) {
        if let Some(child) = child {
            self.push(child);
        }
    }

    fn present_range(&mut self, role: &'static str, range: Option<Range<usize>>) {
        self.present(range.map(|range| range_node(role, range)));
    }

    fn into_tree(self) -> Tree<String> {
        Tree::new(self.label).with_leaves(self.children.into_iter().map(Self::into_tree))
    }
}

fn range_node(role: &'static str, range: Range<usize>) -> Node {
    Node::new(Label::new().word(role).range(range).finish())
}

fn document(doc: &ParsedDocument) -> Node {
    Node::branch(
        "document",
        [
            Node::branch("tokens", doc.tokens().map(|token| token_node(doc, token))),
            Node::branch("diagnostics", doc.diagnostics().iter().map(diagnostic_node)),
            Node::branch(
                "recovery",
                doc.recovery()
                    .map(|observation| recovery_node(doc, observation)),
            ),
            Node::branch("rules", doc.rules().map(rule_node)),
        ],
    )
}

fn token_node(doc: &ParsedDocument, token: SyntaxToken) -> Node {
    let range = token.range();
    Node::new(
        Label::new()
            .range(range.clone())
            .debug(token.kind())
            .lexeme(doc.slice(range))
            .finish(),
    )
}

fn diagnostic_node(diagnostic: &SyntaxDiagnostic) -> Node {
    Node::new(
        Label::new()
            .range(diagnostic.range())
            .debug(diagnostic.kind())
            .finish(),
    )
}

fn recovery_node(doc: &ParsedDocument, observation: &RecoveryObservation) -> Node {
    let range = observation.range();
    Node::new(
        Label::new()
            .range(range.clone())
            .debug(observation.kind())
            .lexeme(doc.slice(range))
            .finish(),
    )
}

fn rule_node(rule: RuleSyntax<'_>) -> Node {
    let name = rule.lhs().and_then(|lhs| lhs.name()).unwrap_or_default();
    let mut node = Node::new(
        Label::new()
            .range(rule.range())
            .word(name)
            .state(rule.structural_state())
            .flag("recovered", rule.has_recovery())
            .finish(),
    );
    node.required("lhs", rule.lhs().map(|lhs| non_terminal_node("lhs", lhs)));
    node.required_range("definition", rule.definition_range());
    node.extend(rule.alternatives().map(alternative_node));
    node.required_range("terminator", rule.terminator_range());
    node
}

fn non_terminal_node(role: &'static str, non_terminal: NonTerminalSyntax<'_>) -> Node {
    let mut node = Node::new(
        Label::new()
            .range(non_terminal.range())
            .word(role)
            .state(non_terminal.structural_state())
            .lexeme(non_terminal.text())
            .finish(),
    );
    node.required(
        "name",
        match (non_terminal.name_range(), non_terminal.name()) {
            (Some(range), Some(name)) => Some(Node::new(
                Label::new().word("name").range(range).lexeme(name).finish(),
            )),
            (None, None) => None,
            _ => unreachable!("name range and text come from the same optional record"),
        },
    );
    node.present_range("type-separator", non_terminal.type_separator_range());
    node.present(non_terminal.ty().map(|ty| string_node("type", ty)));
    node.required_range("closing", non_terminal.closing_delimiter_range());
    node
}

fn alternative_node(alternative: AlternativeSyntax<'_>) -> Node {
    let mut node = Node::new(
        Label::new()
            .range(alternative.range())
            .word("alt")
            .state(alternative.structural_state())
            .flag("recovered", alternative.has_recovery())
            .finish(),
    );
    node.present(
        alternative
            .weight()
            .map(|weight| integer_node("weight", weight)),
    );
    node.extend(alternative.symbols().map(symbol_node));
    node.present(alternative.repeat().map(repeat_node));
    node
}

fn symbol_node(symbol: SymbolSyntax<'_>) -> Node {
    match symbol.kind() {
        SymbolKind::Terminal => string_node(
            "Terminal",
            symbol
                .as_terminal()
                .expect("Terminal kind has a string-literal view"),
        ),
        SymbolKind::NonTerminal => non_terminal_node(
            "NonTerminal",
            symbol
                .as_non_terminal()
                .expect("NonTerminal kind has a non-terminal view"),
        ),
        SymbolKind::Regex => regex_node(
            symbol
                .as_regex()
                .expect("Regex kind has a regular-expression view"),
        ),
    }
}

fn string_node(role: &'static str, string: StringSyntax<'_>) -> Node {
    Node::new(
        Label::new()
            .range(string.range())
            .word(role)
            .word(if string.is_terminated() {
                "complete"
            } else {
                "unterminated"
            })
            .lexeme(string.text())
            .finish(),
    )
}

fn integer_node(role: &'static str, integer: IntegerSyntax<'_>) -> Node {
    Node::new(
        Label::new()
            .range(integer.range())
            .word(role)
            .lexeme(integer.text())
            .finish(),
    )
}

fn regex_node(regex: RegexSyntax<'_>) -> Node {
    let mut node = Node::new(
        Label::new()
            .range(regex.range())
            .word("Regex")
            .state(regex.structural_state())
            .lexeme(regex.text())
            .finish(),
    );
    node.required_range("opening", regex.opening_parenthesis_range());
    node.required(
        "pattern",
        regex
            .pattern()
            .map(|pattern| string_node("pattern", pattern)),
    );
    node.required_range("closing", regex.closing_parenthesis_range());
    node
}

fn repeat_node(repeat: RepeatSyntax<'_>) -> Node {
    let mut node = Node::new(
        Label::new()
            .range(repeat.range())
            .word("repeat")
            .state(repeat.structural_state())
            .lexeme(repeat.text())
            .finish(),
    );
    node.required(
        "lower",
        repeat
            .lower_bound()
            .map(|lower| integer_node("lower", lower)),
    );
    node.present_range("comma", repeat.comma_range());
    node.present(
        repeat
            .upper_bound()
            .map(|upper| integer_node("upper", upper)),
    );
    node.required_range("closing", repeat.closing_delimiter_range());
    node
}
