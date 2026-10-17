//! Grammar productions. Each production returns its record and reports
//! missing syntax. Unexpected source goes to the nearest owner's `skipped`
//! list, using the stop sets below to decide where the owner resumes.

use super::{Parser, Progress, TokenSet};
use crate::error::ExpectedSyntax;
use crate::records::{
    AlternativeRecord, NonTerminalRecord, RegexRecord, RepeatRecord, RuleRecord, Skipped,
    SourceFile, StringRecord, SymbolRecord,
};
use crate::token::TokenKind;

/// Tokens that end one alternative in a rule body. A rule header and the end
/// of the file also end it.
const ALTERNATIVE_STOP: TokenSet = TokenSet::new(&[TokenKind::Or, TokenKind::Semi]);

impl Parser<'_> {
    pub(super) fn source_file(&mut self) -> SourceFile {
        let mut rules = Vec::new();
        let mut skipped = Vec::new();
        let mut progress = Progress::default();
        while !self.at_eof() {
            progress.assert_progressing(self);
            if self.at(TokenKind::LAngle) {
                rules.push(self.rule());
            } else {
                self.skip_one(&mut skipped);
            }
        }
        SourceFile { rules, skipped }
    }

    fn rule(&mut self) -> RuleRecord {
        let start = self.current_start();
        let lhs = self.non_terminal();
        let definition = self.eat_range(TokenKind::Def);
        if definition.is_none() {
            self.missing(ExpectedSyntax::Definition);
        }

        let mut skipped = Vec::new();
        let alts = self.rule_body(&mut skipped);

        let terminator = self.eat_range(TokenKind::Semi);
        if terminator.is_none() {
            self.missing(ExpectedSyntax::RuleTerminator);
        }

        RuleRecord {
            span: start..self.previous_end(start),
            lhs,
            definition,
            alts,
            terminator,
            skipped,
        }
    }

    /// The alternatives of a rule, separated by `|`. Unexpected tokens between
    /// alternatives are recorded in `skipped`.
    fn rule_body(&mut self, skipped: &mut Vec<Skipped>) -> Vec<AlternativeRecord> {
        let mut alts = Vec::new();
        if self.at(TokenKind::Semi) || self.at_eof() || self.at_rule_start() {
            self.missing(ExpectedSyntax::Alternative);
            return alts;
        }

        let mut progress = Progress::default();
        loop {
            progress.assert_progressing(self);
            alts.push(self.alternative());
            self.skip_until(ALTERNATIVE_STOP, skipped);
            if !self.eat(TokenKind::Or) {
                break;
            }
        }
        alts
    }

    fn alternative(&mut self) -> AlternativeRecord {
        let start = self.current_start();
        let weight = self.eat_range(TokenKind::Int);

        let mut symbols = Vec::new();
        while self.at_symbol_start() && !self.at_rule_start() {
            symbols.push(self.symbol());
        }

        let repeat = self.at(TokenKind::LBrace).then(|| self.repeat());

        // An alternative with no symbols is reported only when it is visibly
        // empty or carries a weight or repeat. A stray token is instead
        // reported as unexpected by the caller's recovery.
        let at_end = self.at(TokenKind::Or)
            || self.at(TokenKind::Semi)
            || self.at_eof()
            || self.at_rule_start();
        if symbols.is_empty() && (weight.is_some() || repeat.is_some() || at_end) {
            self.missing(ExpectedSyntax::Symbol);
        }

        AlternativeRecord {
            span: start..self.previous_end(start),
            weight,
            symbols,
            repeat,
        }
    }

    fn symbol(&mut self) -> SymbolRecord {
        if self.at_string() {
            SymbolRecord::Terminal(self.string())
        } else if self.at(TokenKind::LAngle) {
            SymbolRecord::NonTerminal(self.non_terminal())
        } else {
            SymbolRecord::Regex(self.regex())
        }
    }

    /// Consumes a string token, which the caller has checked is present.
    fn string(&mut self) -> StringRecord {
        let terminated = self.at(TokenKind::Str);
        let span = self.bump();
        StringRecord { span, terminated }
    }

    /// Consumes a string token if one is present; otherwise reports `expected`.
    fn optional_string(&mut self, expected: ExpectedSyntax) -> Option<StringRecord> {
        if self.at_string() {
            Some(self.string())
        } else {
            self.missing(expected);
            None
        }
    }

    fn non_terminal(&mut self) -> NonTerminalRecord {
        let opening = self.bump(); // the caller established `<`
        let start = opening.start;
        let name = self.eat_range(TokenKind::Id);
        if name.is_none() {
            self.missing(ExpectedSyntax::NonTerminalName);
        }
        let type_separator = self.eat_range(TokenKind::Colon);
        let ty = if type_separator.is_some() {
            self.optional_string(ExpectedSyntax::NonTerminalType)
        } else {
            None
        };
        let closing = self.eat_range(TokenKind::RAngle);
        if closing.is_none() {
            self.missing(ExpectedSyntax::NonTerminalClosingDelimiter);
        }
        NonTerminalRecord {
            span: start..self.previous_end(start),
            opening,
            name,
            type_separator,
            ty,
            closing,
        }
    }

    fn regex(&mut self) -> RegexRecord {
        let start = self.bump().start; // `re`
        let opening_parenthesis = self.eat_range(TokenKind::LParen);
        if opening_parenthesis.is_none() {
            self.missing(ExpectedSyntax::RegexOpeningParenthesis);
        }
        let pattern = if opening_parenthesis.is_some() {
            self.optional_string(ExpectedSyntax::RegexPattern)
        } else {
            None
        };
        let closing_parenthesis = self.eat_range(TokenKind::RParen);
        if closing_parenthesis.is_none() {
            self.missing(ExpectedSyntax::RegexClosingParenthesis);
        }
        RegexRecord {
            span: start..self.previous_end(start),
            opening_parenthesis,
            pattern,
            closing_parenthesis,
        }
    }

    fn repeat(&mut self) -> RepeatRecord {
        let opening = self.bump(); // the caller established `{`
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
        RepeatRecord {
            span: start..self.previous_end(start),
            opening,
            lower,
            comma,
            upper,
            closing,
        }
    }
}
