use crate::{
    ast::{Block, MarstonDocument, Node, Value, ident_table::get_or_intern},
    context::Context,
    error_report,
    lexer::{Token, TokenKind},
    reports::ReportsBag,
    span::SpanUtils,
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
    pub doc: MarstonDocument,
}

impl<'a> Parser<'a> {
    pub fn new(ctx: &'a Context, tokens: Vec<Token>) -> Self {
        Self { ctx, tokens, current: 0, doc: MarstonDocument::new() }
    }

    pub fn parse(&mut self) {
        while !self.is_at_end() {
            let block = self.parse_block();
            self.doc.add_block(block);

            if ReportsBag::has_errors() {
                break;
            }
        }
    }

    pub fn parse_block(&mut self) -> Block {
        let dot = self.consume(&TokenKind::Dot, "Blocks are required to start with a dot").cloned();

        let mut attrs: FxHashMap<Spur, Value> = Default::default();

        let identifier = self.consume_identifier("Expected block name");
        let mut block = Block::new(identifier);

        // Parse inline attributes in parentheses
        if self.check(&TokenKind::ParenOpen) {
            self.advance();

            while !self.check(&TokenKind::ParenClose) && !self.is_at_end() {
                let attr = self.parse_attr();

                if self.current().kind != TokenKind::ParenClose {
                    // Use the previous token's end as the error span for missing comma
                    let error_span = self
                        .previous()
                        .map(|t| t.span.to_end())
                        .unwrap_or_else(|| self.current().span.clone());

                    self.consume_with_span(
                        &TokenKind::Comma,
                        "Attributes must be separated by commas",
                        error_span,
                    );
                }

                if let Some((attr_name, attr_value)) = attr {
                    attrs.insert(attr_name, attr_value);
                } else {
                    debug!("found invalid attribute list");
                    break;
                }
            }

            self.consume(
                &TokenKind::ParenClose,
                "Block's inner attribute list is missing a closing parenthesis",
            );
        }

        if self.check(&TokenKind::BraceOpen) {
            self.advance();

            let mut children = Vec::new();

            while !self.check(&TokenKind::BraceClose) && !self.is_at_end() {
                if self.current().kind == TokenKind::Dot {
                    if let Some(ahead) = self.peek_ahead(2) {
                        match ahead.kind {
                            TokenKind::Equals => {
                                let attr = self.parse_attr();
                                if let Some((attr_name, attr_value)) = attr {
                                    attrs.insert(attr_name, attr_value);
                                }
                            }
                            _ => {
                                let child = self.parse_block();
                                children.push(Node::Block(child));
                            }
                        }

                        self.match_token(&TokenKind::Comma);
                    } else {
                        self.error_at_current("Unexpected end of input after '.'");
                        break;
                    }
                } else if let TokenKind::String(string) = &self.current().kind {
                    children.push(Node::Text(string.clone()));
                    self.advance();
                    self.match_token(&TokenKind::Comma);
                } else {
                    self.error_at_current(
                        "Invalid block children. Expected a block, an attribute, or content",
                    );
                    self.advance();
                }
            }

            self.consume(&TokenKind::BraceClose, "Blocks should end in a brace");

            if !children.is_empty() {
                block.children = children;
            }
        }
        if !attrs.is_empty() {
            block.attributes = attrs;
        }

        if let Some(dot) = dot
            && let Some(previous) = self.previous()
        {
            block.span = dot.span.to(previous.span.clone());
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
            self.error_expected_token(token_type, message);
            None
        }
    }

    /// Consume a token with a custom error span
    pub fn consume_with_span(
        &mut self,
        token_type: &TokenKind,
        message: &str,
        span: Range<usize>,
    ) -> Option<&Token> {
        if self.check(token_type) {
            self.advance()
        } else {
            self.error_expected_token_at_span(token_type, message, span);
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

    /// Consume any of the expected token types with a custom error span
    pub fn consume_any_with_span(
        &mut self,
        token_types: &[TokenKind],
        message: &str,
        span: Range<usize>,
    ) -> Option<&Token> {
        if self.check_any(token_types) {
            self.advance()
        } else {
            self.error_expected_any_token_at_span(token_types, message, span);
            None
        }
    }

    /// Consume an identifier token specifically
    pub fn consume_identifier(&mut self, message: &str) -> Option<Spur> {
        if let TokenKind::Identifier(name) = &self.current().kind {
            let interned = get_or_intern(name);
            self.advance();
            return Some(interned);
        }

        self.error_expected_identifier(message);
        None
    }

    /// Consume an identifier with a custom error span
    pub fn consume_identifier_with_span(
        &mut self,
        message: &str,
        span: Range<usize>,
    ) -> Option<Spur> {
        if let TokenKind::Identifier(name) = &self.current().kind {
            let interned = get_or_intern(name);
            self.advance();
            return Some(interned);
        }

        self.error_expected_identifier_at_span(message, span);
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

    // Error reporting methods
    fn error_expected_token(&mut self, expected: &TokenKind, message: &str) {
        let error_msg = format!("Expected '{}', found '{}'", expected, self.current().kind);
        self.error_at_current(&format!("{}. {}", error_msg, message));
    }

    fn error_expected_token_at_span(
        &mut self,
        expected: &TokenKind,
        message: &str,
        span: Range<usize>,
    ) {
        let error_msg = format!("Expected '{}', found '{}'", expected, self.current().kind);
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

    fn error_expected_any_token_at_span(
        &mut self,
        expected: &[TokenKind],
        message: &str,
        span: Range<usize>,
    ) {
        let expected_names: Vec<String> = expected.iter().map(|t| format!("'{}'", t)).collect();
        let error_msg = format!(
            "Expected one of [{}], found '{}'",
            expected_names.join(", "),
            self.current().kind
        );
        self.error_at(&format!("{}. {}", error_msg, message), span);
    }

    fn error_expected_identifier(&mut self, message: &str) {
        self.error_at_current(&format!(
            "Expected identifier, found '{}'. {}",
            self.current(),
            message
        ));
    }

    fn error_expected_identifier_at_span(&mut self, message: &str, span: Range<usize>) {
        self.error_at(
            &format!("Expected identifier, found '{}'. {}", self.current(), message),
            span,
        );
    }

    // Core error reporting methods - these remain unchanged for at_current/at_previous usage
    pub fn error_at_current(&mut self, message: &str) {
        let token = self.current();
        ReportsBag::add(error_report!(
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
        ReportsBag::add(error_report!(
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
            ReportsBag::add(error_report!(
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

        ReportsBag::add(error_report!(
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
        label_span: Range<usize>,
        label_message: &str,
    ) {
        ReportsBag::add(error_report!(
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
        ReportsBag::add(error_report!(
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
            span: primary_span,
            message: message,
            labels: {}
        );

        ReportsBag::add(report);
    }

    /// Check if the current position matches a sequence of tokens
    pub fn check_sequence(&self, sequence: &[TokenKind]) -> bool {
        sequence.iter().enumerate().all(|(i, expected)| {
            self.peek_ahead(i).map(|token| token.kind == *expected).unwrap_or(false)
        })
    }
}
