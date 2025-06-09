use logos::{Lexer, Logos, Span};
use std::fmt;
use std::fmt::{Display, Formatter};
use std::ops::Range;

#[derive(Debug, Logos, Clone, PartialEq)]
#[logos(skip r"[ \t\r\n\f]+")]
#[logos(skip r"//[^\n]*")]
#[logos(skip r"/\*[^*]*\*+([^/*][^*]*\*+)*/")]
pub enum TokenKind {
    #[token("false", |_| false)]
    #[token("true", |_| true)]
    Bool(bool),

    #[token("{")]
    BraceOpen,

    #[token("}")]
    BraceClose,

    #[token("[")]
    BracketOpen,

    #[token("]")]
    BracketClose,

    #[token("(")]
    ParenOpen,

    #[token(")")]
    ParenClose,

    #[token(".")]
    Dot,

    #[token("=")]
    Equals,

    #[token(",")]
    Comma,

    #[regex(r"-?(?:0|[1-9]\d*)(?:\.\d+)?(?:[eE][+-]?\d+)?", |lex| lex.slice().parse::<f64>().unwrap())]
    Number(f64),

    #[regex(r#""([^"\\\x00-\x1F]|\\(["\\bnfrt/]|u[a-fA-F0-9]{4}))*""#, |lex| lex.slice().to_owned())]
    String(String),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_owned())]
    Identifier(String),
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        use TokenKind::*;

        match self {
            Bool(true) => write!(f, "true"),
            Bool(false) => write!(f, "false"),
            BraceOpen => write!(f, "{{"),
            BraceClose => write!(f, "}}"),
            BracketOpen => write!(f, "["),
            BracketClose => write!(f, "]"),
            ParenOpen => write!(f, "("),
            ParenClose => write!(f, ")"),
            Dot => write!(f, "."),
            Equals => write!(f, "="),
            Comma => write!(f, ","),
            Number(n) => write!(f, "{}", n),
            String(s) => write!(f, "{}", s),
            Identifier(ident) => write!(f, "{}", ident),
        }
    }
}

impl TokenKind {
    pub fn get_tokens(input: &str) -> Vec<Token> {
        let mut lexer = TokenKind::lexer(input);
        let mut tokens = Vec::new();

        while let Some(token) = lexer.next() {
            if let Ok(token) = token {
                tokens.push(Token { kind: token, span: lexer.span() });
            }
        }

        tokens
    }
}

#[derive(Debug, Clone)]
pub struct Token {
    pub kind: TokenKind,
    pub span: Range<usize>,
}

impl Display for Token {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.kind)
    }
}

impl PartialEq for Token {
    fn eq(&self, other: &Self) -> bool {
        self.kind == other.kind
    }
}
