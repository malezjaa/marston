use logos::{Lexer, Logos, Span};

#[derive(Debug, Logos)]
#[logos(skip r"[ \t\r\n\f]+")]
pub enum Token {
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

impl Token {
    pub fn get_tokens(str: &str) -> Vec<Token> {
        Self::lexer(str).filter_map(|op| op.ok()).collect::<Vec<_>>()
    }
}
