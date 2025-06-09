use crate::{
    ast::{Block, MarstonDocument, Value, ident_table::intern},
    context::Context,
    error_report,
    lexer::{Token, TokenKind},
    reports::TemporaryBag,
};
use ariadne::{Color, Label, Report, ReportKind};
use lasso::Spur;
use log::debug;
use rustc_hash::FxHashMap;
use std::ops::Range;

#[derive(Debug)]
pub struct Parser<'a> {
    pub ctx: &'a Context,
    pub tokens: Vec<Token>,
    pub current: usize,
    pub bag: TemporaryBag<'a>,
    pub doc: MarstonDocument,
}

impl<'a> Parser<'a> {
    pub fn new(ctx: &'a Context, tokens: Vec<Token>) -> Self {
        Self { ctx, tokens, current: 0, bag: TemporaryBag::new(), doc: MarstonDocument::new() }
    }

    pub fn parse(&mut self) {
        while !self.is_at_end() {
            let block = self.parse_block();
            self.doc.add_block(block);

            if self.bag.has_errors() {
                break;
            }
        }
    }

    pub fn parse_block(&mut self) -> Block {
        self.consume(&TokenKind::Dot, "Blocks are required to start with a dot");

        let mut attrs: FxHashMap<Spur, Value> = Default::default();

        let identifier = self.consume_identifier("Expected block name");
        let block = Block::new(identifier);
        if self.check(&TokenKind::ParenOpen) {
            self.advance();

            while !self.check(&TokenKind::ParenClose) {
                let attr = self.parse_attr();

                if self.current().kind != TokenKind::ParenClose {
                    self.consume_on_earlier_span(
                        &TokenKind::Comma,
                        "Attributes must be separated by commas",
                    );
                }

                if let Some((attr_name, attr_value)) = attr {
                    attrs.insert(attr_name, attr_value);
                } else {
                    debug!("found invalid attribute list");
                    break;
                }
            }

            self.advance();
        }

        block
    }

    pub fn parse_attr(&mut self) -> Option<(Spur, Value)> {
        let _dot = self.consume(&TokenKind::Dot, "Attributes are required to start with a dot")?;
        let identifier = self.consume_identifier("Expected attribute name")?;
        self.consume(&TokenKind::Equals, "attribute name must be separated by an equals sign")?;
        let value = self.parse_value()?;

        Some((identifier, value))
    }

    pub fn parse_value(&mut self) -> Option<Value> {
        let val = match &self.current().kind {
            TokenKind::String(s) => Some(Value::String(s.clone())),
            TokenKind::Number(num) => Some(Value::Number(*num)),
            TokenKind::Bool(bool) => Some(Value::Boolean(*bool)),

            TokenKind::BracketOpen => {
                self.advance();
                let mut values = Vec::new();

                if !self.check(&TokenKind::BracketClose) {
                    loop {
                        if let Some(value) = self.parse_value() {
                            values.push(value);
                        } else {
                            break;
                        }

                        if self.match_token(&TokenKind::Comma) {
                            continue;
                        }

                        break;
                    }
                }

                if !self
                    .consume(&TokenKind::BracketClose, "Expected closing bracket after array")
                    .is_some()
                {
                    return None;
                }

                Some(Value::Array(values))
            }

            _ => {
                self.error_at_current(
                    "expected the value to be one of: string, boolean, number, array.",
                );
                None
            }
        };

        if !matches!(val, Some(Value::Array(_))) {
            self.advance();
        }

        val
    }

    pub fn current(&self) -> &Token {
        &self.tokens[self.current]
    }

    pub fn advance(&mut self) -> Option<&Token> {
        if !self.is_at_end() {
            let token = &self.tokens[self.current];
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

    pub fn peek_ahead(&self, offset: usize) -> Option<&Token> {
        self.tokens.get(self.current + offset)
    }

    pub fn previous(&self) -> Option<&Token> {
        if self.current > 0 { self.tokens.get(self.current - 1) } else { None }
    }

    pub fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    pub fn check(&self, token_type: &TokenKind) -> bool {
        self.current().kind == *token_type
    }

    /// Check if the current token matches any of the given types
    pub fn check_any(&self, token_types: &[TokenKind]) -> bool {
        token_types.iter().any(|t| self.check(t))
    }

    /// Advance if the current token matches the type
    pub fn match_token(&mut self, token_type: &TokenKind) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Advance if the current token matches any of the types
    pub fn match_any(&mut self, token_types: &[TokenKind]) -> bool {
        if self.check_any(token_types) {
            self.advance();
            true
        } else {
            false
        }
    }

    /// Consume a token of the expected type or report an error
    pub fn consume(&mut self, token_type: &TokenKind, message: &str) -> Option<&Token> {
        if self.check(token_type) {
            self.advance()
        } else {
            self.error_expected_token(token_type, message, false);
            None
        }
    }

    pub fn consume_on_earlier_span(
        &mut self,
        token_type: &TokenKind,
        message: &str,
    ) -> Option<&Token> {
        if self.check(token_type) {
            self.advance()
        } else {
            self.error_expected_token(token_type, message, true);
            None
        }
    }

    /// Consume any of the expected token types or report an error
    pub fn consume_any(&mut self, token_types: &[TokenKind], message: &str) -> Option<&Token> {
        if self.check_any(token_types) {
            self.advance()
        } else {
            self.error_expected_any_token(token_types, message);
            None
        }
    }

    /// Consume an identifier token specifically
    pub fn consume_identifier(&mut self, message: &str) -> Option<Spur> {
        if let TokenKind::Identifier(name) = &self.current().kind {
            let interned = intern(name);
            self.advance();
            return Some(interned);
        }

        self.error_expected_identifier(message);
        None
    }

    /// Consume a specific identifier by name
    pub fn consume_specific_identifier(&mut self, expected: &str, message: &str) -> bool {
        if let TokenKind::Identifier(name) = &self.current().kind {
            if name == expected {
                self.advance();
                return true;
            }
        }

        self.error_at_current(&format!("Expected identifier '{}'. {}", expected, message));
        false
    }

    pub fn skip_until(&mut self, token_types: &[TokenKind]) -> bool {
        while !self.is_at_end() && !self.check_any(token_types) {
            self.advance();
        }
        !self.is_at_end()
    }

    fn error_expected_token(&mut self, expected: &TokenKind, message: &str, earlier_span: bool) {
        let error_msg = format!("Expected '{}', found '{}'", expected, self.current().kind);
        let current = self.current().span.clone();
        let span = if earlier_span { current.start - 1..current.start - 1 } else { current };

        self.error_at(&format!("{}. {}", error_msg, message), span);
    }

    fn error_expected_any_token(&mut self, expected: &[TokenKind], message: &str) {
        let expected_names: Vec<String> = expected.iter().map(|t| format!("'{}'", t)).collect();

        let error_msg = format!(
            "Expected one of [{}], found '{}'",
            expected_names.join(", "),
            self.current().kind
        );
        self.error_at_current(&format!("{}. {}", error_msg, message));
    }

    fn error_expected_identifier(&mut self, message: &str) {
        self.error_at_current(&format!(
            "Expected identifier, found '{}'. {}",
            self.current(),
            message
        ));
    }

    pub fn error_at_current(&mut self, message: &str) {
        let token = self.current();
        self.bag.add(error_report!(
            file: self.ctx.file(),
            span: token.span.clone(),
            message: message,
            labels: {
                token.span.clone() => {
                    message: message => Color::BrightRed
                }
            }
        ));
    }

    pub fn error_at(&mut self, message: &str, span: Range<usize>) {
        self.bag.add(error_report!(
            file: self.ctx.file(),
            span: span.clone(),
            message: message,
            labels: {
                span => {
                    message: message => Color::BrightRed
                }
            }
        ));
    }

    pub fn error_at_previous(&mut self, message: &str) {
        if let Some(token) = self.previous() {
            self.bag.add(error_report!(
                file: self.ctx.file(),
                span: token.span.clone(),
                message: message,
                labels: {
                    token.span.clone() => {
                        message: message => Color::BrightRed
                    }
                }
            ));
        } else {
            self.error_at_end(message);
        }
    }

    fn error_at_end(&mut self, message: &str) {
        let end_span =
            self.tokens.last().map(|token| token.span.end..token.span.end).unwrap_or(0..0);

        self.bag.add(error_report!(
            file: self.ctx.file(),
            span: end_span.clone(),
            message: format!("Unexpected end of input: {}", message),
            labels: {
                end_span => {
                    message: "unexpected end of input" => Color::BrightRed
                }
            }
        ));
    }

    pub fn error_with_label(
        &mut self,
        message: &str,
        label_span: std::ops::Range<usize>,
        label_message: &str,
    ) {
        self.bag.add(error_report!(
            file: self.ctx.file(),
            span: label_span.clone(),
            message: message,
            labels: {
                label_span => {
                    message: label_message => Color::BrightRed
                }
            }
        ));
    }

    pub fn error_with_note(&mut self, message: &str, span: Range<usize>, note: &str) {
        self.bag.add(error_report!(
            file: self.ctx.file(),
            span: span,
            message: message,
            notes: [note]
        ));
    }

    pub fn error_with_multiple_labels(
        &mut self,
        message: &str,
        labels: &[(Range<usize>, &str, Color)],
    ) {
        let primary_span = labels.first().map(|(span, _, _)| span.clone()).unwrap_or(0..0);

        let mut report = error_report!(
            file: self.ctx.file(),
            span: primary_span,
            message: message,
            labels: {}
        );

        self.bag.add(report);
    }

    /// Check if the current position matches a sequence of tokens
    pub fn check_sequence(&self, sequence: &[TokenKind]) -> bool {
        sequence.iter().enumerate().all(|(i, expected)| {
            self.peek_ahead(i).map(|token| token.kind == *expected).unwrap_or(false)
        })
    }

    /// Get the span from start token to current position
    pub fn span_from(&self, start_token: &Token) -> Range<usize> {
        let end = self
            .previous()
            .map(|t| t.span.end)
            .or_else(|| Some(self.current().span.start))
            .unwrap_or(start_token.span.end);

        start_token.span.start..end
    }

    /// Create a span covering multiple tokens
    pub fn span_between(&self, start: &Token, end: &Token) -> Range<usize> {
        start.span.start..end.span.end
    }
}
