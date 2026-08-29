//! One private tolerant recursive-descent parser over significant tokens.
//!
//! The parser is deliberately responsible for both ordinary recognition and
//! recovery. It records source-established facts in range-backed records,
//! reports missing syntax without inventing source tokens, and consumes
//! unexpected source only when doing so advances the cursor.

use std::ops::Range;

use crate::error::{ExpectedSyntax, SyntaxDiagnostic, SyntaxDiagnosticKind};
use crate::records::{
    AlternativeRecord, NonTerminalRecord, RegexRecord, RepeatRecord, RuleRecord, StringRecord,
    StructuralState, SymbolRecord,
};
use crate::recovery::{RecoveryKind, RecoveryObservation};
use crate::token::{SyntaxToken, TokenKind};

pub(crate) struct ParseOutput {
    pub(crate) rules: Vec<RuleRecord>,
    pub(crate) diagnostics: Vec<SyntaxDiagnostic>,
    pub(crate) recovery: Vec<RecoveryObservation>,
}

pub(crate) fn parse(
    tokens: &[SyntaxToken],
    lexical_diagnostics: Vec<SyntaxDiagnostic>,
) -> ParseOutput {
    let mut parser = Parser {
        tokens: tokens
            .iter()
            .filter(|token| token.kind().is_significant())
            .collect(),
        cursor: 0,
        diagnostics: lexical_diagnostics,
        recovery: Vec::new(),
    };
    parser.record_lexical_recovery(tokens);

    let mut rules = Vec::new();
    while !parser.at_eof() {
        if parser.at(TokenKind::LAngle) {
            rules.push(parser.parse_rule());
        } else {
            parser.unexpected();
        }
    }

    ParseOutput {
        rules,
        diagnostics: parser.diagnostics,
        recovery: parser.recovery,
    }
}

struct Parser<'a> {
    tokens: Vec<&'a SyntaxToken>,
    cursor: usize,
    diagnostics: Vec<SyntaxDiagnostic>,
    recovery: Vec<RecoveryObservation>,
}

impl<'a> Parser<'a> {
    fn record_lexical_recovery(&mut self, tokens: &[SyntaxToken]) {
        for token in tokens {
            let kind = match token.kind() {
                TokenKind::Invalid => Some(RecoveryKind::InvalidLexeme),
                TokenKind::UnterminatedStr => Some(RecoveryKind::UnterminatedString),
                _ => None,
            };
            if let Some(kind) = kind {
                self.recovery
                    .push(RecoveryObservation::new(token.range(), kind));
            }
        }
    }

    fn parse_rule(&mut self) -> RuleRecord {
        let start = self.current_start();
        let lhs = self.parse_non_terminal();
        let definition = self.eat_range(TokenKind::Def);
        if definition.is_none() {
            self.missing(ExpectedSyntax::Definition);
        }

        let mut alts = Vec::new();
        if definition.is_some() {
            loop {
                if self.at(TokenKind::Semi) || self.at_eof() || self.looks_like_rule_start() {
                    if alts.is_empty() {
                        self.missing(ExpectedSyntax::Alternative);
                    }
                    break;
                }
                if let Some(alternative) = self.parse_alternative() {
                    alts.push(alternative);
                } else if !self.at_eof() {
                    self.unexpected();
                }
                if self.eat(TokenKind::Or) {
                    continue;
                }
                break;
            }
        }

        let terminator = self.eat_range(TokenKind::Semi);
        if terminator.is_none() {
            self.missing(ExpectedSyntax::RuleTerminator);
        }
        let end = terminator
            .as_ref()
            .map_or_else(|| self.previous_end_or(start), |range| range.end);
        let state = if lhs.state == StructuralState::Complete
            && definition.is_some()
            && !alts.is_empty()
            && terminator.is_some()
        {
            StructuralState::Complete
        } else {
            StructuralState::Partial
        };
        let span = start..end;
        let has_recovery = self
            .recovery
            .iter()
            .any(|recovery| overlaps(&span, &recovery.range()));
        RuleRecord {
            span,
            lhs: Some(lhs),
            definition,
            alts,
            terminator,
            state,
            has_recovery,
        }
    }

    fn parse_non_terminal(&mut self) -> NonTerminalRecord {
        let opening = self.bump_range(); // caller established `<`
        let start = opening.start;
        let name = self.eat_range(TokenKind::Id);
        if name.is_none() {
            self.missing(ExpectedSyntax::NonTerminalName);
        }
        let type_separator = self.eat_range(TokenKind::Colon);
        let ty = if type_separator.is_some() {
            self.parse_string()
        } else {
            None
        };
        let closing = self.eat_range(TokenKind::RAngle);
        if closing.is_none() {
            self.missing(ExpectedSyntax::NonTerminalClosingDelimiter);
        }
        let end = closing
            .as_ref()
            .map_or_else(|| self.previous_end_or(opening.end), |range| range.end);
        let state = if name.is_some()
            && closing.is_some()
            && (type_separator.is_none() || ty.as_ref().is_some_and(|string| string.terminated))
        {
            StructuralState::Complete
        } else {
            StructuralState::Partial
        };
        NonTerminalRecord {
            span: start..end,
            opening,
            name,
            type_separator,
            ty,
            closing,
            state,
        }
    }

    fn parse_alternative(&mut self) -> Option<AlternativeRecord> {
        let start = self.current_start();
        let weight = self.eat_range(TokenKind::Int);
        let mut symbols = Vec::new();
        while !self.at_eof()
            && !self.at(TokenKind::Or)
            && !self.at(TokenKind::Semi)
            && !self.at(TokenKind::LBrace)
            && !self.looks_like_rule_start()
        {
            match self.parse_symbol() {
                Some(symbol) => symbols.push(symbol),
                None => break,
            }
        }
        let repeat = if self.at(TokenKind::LBrace) {
            Some(self.parse_repeat())
        } else {
            None
        };
        if symbols.is_empty() {
            if weight.is_some() {
                self.missing(ExpectedSyntax::Symbol);
            }
            return None;
        }
        let end = repeat
            .as_ref()
            .map_or_else(|| self.previous_end_or(start), |repeat| repeat.span.end);
        let span = start..end;
        let state = if symbols
            .iter()
            .all(|symbol| symbol.state() == StructuralState::Complete)
            && repeat
                .as_ref()
                .is_none_or(|repeat| repeat.state == StructuralState::Complete)
        {
            StructuralState::Complete
        } else {
            StructuralState::Partial
        };
        let has_recovery = self
            .recovery
            .iter()
            .any(|recovery| overlaps(&span, &recovery.range()));
        Some(AlternativeRecord {
            span,
            weight,
            symbols,
            repeat,
            state,
            has_recovery,
        })
    }

    fn parse_symbol(&mut self) -> Option<SymbolRecord> {
        match self.current_kind() {
            Some(TokenKind::Str) | Some(TokenKind::UnterminatedStr) => {
                self.parse_string().map(SymbolRecord::Terminal)
            }
            Some(TokenKind::LAngle) => Some(SymbolRecord::NonTerminal(self.parse_non_terminal())),
            Some(TokenKind::Re) => Some(SymbolRecord::Regex(self.parse_regex())),
            _ => None,
        }
    }

    fn parse_string(&mut self) -> Option<StringRecord> {
        let terminated = self.at(TokenKind::Str);
        if !terminated && !self.at(TokenKind::UnterminatedStr) {
            self.missing(ExpectedSyntax::RegexPattern);
            return None;
        }
        let span = self.bump_range();
        Some(StringRecord { span, terminated })
    }

    fn parse_regex(&mut self) -> RegexRecord {
        let start = self.bump_range().start; // `re`
        let opening_parenthesis = self.eat_range(TokenKind::LParen);
        if opening_parenthesis.is_none() {
            self.missing(ExpectedSyntax::RegexOpeningParenthesis);
        }
        let pattern = if opening_parenthesis.is_some() {
            self.parse_string()
        } else {
            None
        };
        let closing_parenthesis = self.eat_range(TokenKind::RParen);
        if closing_parenthesis.is_none() {
            self.missing(ExpectedSyntax::RegexClosingParenthesis);
        }
        let end = closing_parenthesis
            .as_ref()
            .map_or_else(|| self.previous_end_or(start), |range| range.end);
        let state = if opening_parenthesis.is_some()
            && pattern.as_ref().is_some_and(|string| string.terminated)
            && closing_parenthesis.is_some()
        {
            StructuralState::Complete
        } else {
            StructuralState::Partial
        };
        RegexRecord {
            span: start..end,
            opening_parenthesis,
            pattern,
            closing_parenthesis,
            state,
        }
    }

    fn parse_repeat(&mut self) -> RepeatRecord {
        let opening = self.bump_range(); // caller established `{`
        let start = opening.start;
        let lower = self.eat_range(TokenKind::Int);
        if lower.is_none() {
            self.missing(ExpectedSyntax::RepeatLowerBound);
        }
        let comma = self.eat_range(TokenKind::Comma);
        let upper = if comma.is_some() {
            self.eat_range(TokenKind::Int)
        } else {
            None
        };
        let closing = self.eat_range(TokenKind::RBrace);
        if closing.is_none() {
            self.missing(ExpectedSyntax::RepeatClosingDelimiter);
        }
        let end = closing
            .as_ref()
            .map_or_else(|| self.previous_end_or(start), |range| range.end);
        let state = if lower.is_some() && closing.is_some() {
            StructuralState::Complete
        } else {
            StructuralState::Partial
        };
        RepeatRecord {
            span: start..end,
            opening,
            lower,
            comma,
            upper,
            closing,
            state,
        }
    }

    fn unexpected(&mut self) {
        let range = self.bump_range();
        self.diagnostics.push(SyntaxDiagnostic::new(
            SyntaxDiagnosticKind::UnexpectedToken,
            range.clone(),
        ));
        self.recovery.push(RecoveryObservation::new(
            range,
            RecoveryKind::UnexpectedToken,
        ));
    }

    fn missing(&mut self, expected: ExpectedSyntax) {
        let offset = self.current_start();
        self.diagnostics.push(SyntaxDiagnostic::new(
            SyntaxDiagnosticKind::Missing(expected),
            offset..offset,
        ));
    }

    fn looks_like_rule_start(&self) -> bool {
        self.at(TokenKind::LAngle)
            && self
                .tokens
                .get(self.cursor + 1)
                .is_some_and(|token| token.kind() == TokenKind::Id)
    }

    fn current_kind(&self) -> Option<TokenKind> {
        self.tokens.get(self.cursor).map(|token| token.kind())
    }

    fn current_start(&self) -> usize {
        self.tokens
            .get(self.cursor)
            .map_or_else(|| self.previous_end_or(0), |token| token.range().start)
    }

    fn previous_end_or(&self, fallback: usize) -> usize {
        self.cursor
            .checked_sub(1)
            .and_then(|index| self.tokens.get(index))
            .map_or(fallback, |token| token.range().end)
    }

    fn at(&self, kind: TokenKind) -> bool {
        self.current_kind() == Some(kind)
    }

    fn at_eof(&self) -> bool {
        self.cursor == self.tokens.len()
    }

    fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.cursor += 1;
            true
        } else {
            false
        }
    }

    fn eat_range(&mut self, kind: TokenKind) -> Option<Range<usize>> {
        if self.at(kind) {
            Some(self.bump_range())
        } else {
            None
        }
    }

    fn bump_range(&mut self) -> Range<usize> {
        let token = self
            .tokens
            .get(self.cursor)
            .expect("parser bumps only at a token");
        self.cursor += 1;
        token.range()
    }
}

fn overlaps(left: &Range<usize>, right: &Range<usize>) -> bool {
    left.start < right.end && right.start < left.end
}
