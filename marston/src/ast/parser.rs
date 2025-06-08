use crate::ast::MarstonDocument;
use crate::context::Context;
use crate::error_report;
use crate::lexer::{Token, TokenKind};
use crate::reports::{ReportsBag, TemporaryBag};
use ariadne::{Color, Label, Report, ReportKind};
use bumpalo::Bump;

#[derive(Debug)]
pub struct Parser<'a> {
    pub ctx: &'a Context,
    pub tokens: Vec<Token>,
    pub bump: Bump,
    pub current: usize,
    pub bag: TemporaryBag<'a>,
    pub doc: MarstonDocument,
}

impl<'a> Parser<'a> {
    pub fn new(ctx: &'a Context, tokens: Vec<Token>) -> Self {
        Self {
            ctx,
            tokens,
            bump: Bump::new(),
            current: 0,
            bag: TemporaryBag::new(),
            doc: MarstonDocument::new(),
        }
    }

    pub fn parse(&mut self) {
        while !self.is_at_end() {
            self.consume(&TokenKind::Dot, "document always expects blocks at top-level");
            break;
        }
    }

    pub fn advance(&mut self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            let token = &self.tokens[self.current];
            self.current += 1;
            Some(token)
        } else {
            None
        }
    }

    pub fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.current)
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
        if let Some(current_token) = self.peek() {
            current_token.kind == *token_type
        } else {
            false
        }
    }

    pub fn match_token(&mut self, token_type: &TokenKind) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    pub fn match_any(&mut self, token_types: &[TokenKind]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    pub fn consume(&mut self, token_type: &TokenKind, message: &str) -> Option<&Token> {
        if self.check(token_type) {
            Some(self.advance().unwrap())
        } else {
            let found = if let Some(current) = self.peek() {
                format!("'{}'", current)
            } else {
                "end of input".to_string()
            };

            let error_msg = format!("Expected '{}', found {}", token_type, found);
            self.error_at_current(&format!("{}. {}", error_msg, message));
            None
        }
    }

    pub fn consume_any(&mut self, token_types: &[TokenKind], message: &str) -> Option<&Token> {
        for token_type in token_types {
            if self.check(token_type) {
                return Some(self.advance().unwrap());
            }
        }

        let expected_names: Vec<String> = token_types.iter().map(|t| format!("{:?}", t)).collect();
        let found = if let Some(current) = self.peek() {
            format!("{:?}", current)
        } else {
            "end of input".to_string()
        };

        let error_msg = format!("Expected one of [{}], found {}", expected_names.join(", "), found);
        self.error_at_current(&format!("{}. {}", error_msg, message));
        None
    }

    pub fn error_at_current(&mut self, message: &str) {
        if let Some(token) = self.peek() {
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
            let end_span = if let Some(last_token) = self.tokens.last() {
                last_token.span.clone()
            } else {
                0..0
            };

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
            self.bag.add(error_report!(
                file: self.ctx.file(),
                span: 0..0,
                message: message,
            ));
        }
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
                    message: label_message => ariadne::Color::BrightRed
                }
            }
        ));
    }

    pub fn error_with_note(&mut self, message: &str, span: std::ops::Range<usize>, note: &str) {
        self.bag.add(error_report!(
            file: self.ctx.file(),
            span: span,
            message: message,
            notes: [note]
        ));
    }
}
