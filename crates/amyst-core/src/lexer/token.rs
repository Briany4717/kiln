use std::fmt::Display;

#[derive(Debug, Clone, PartialEq)]
pub struct Token<'a> {
    pub token_type: TokenType<'a>,
    pub lexeme: &'a str,
    pub line: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType<'a> {
    LeftParen,
    RightParen,
    LeftBrace,
    RightBrace,
    Comma,
    Arrow,
    Dot,
    DotDot,
    DotDotEqual,
    Minus,
    Plus,
    Semicolon,
    Colon,
    Slash,
    Star,
    Bang,
    BangEqual,
    Equal,
    EqualEqual,
    Greater,
    GreaterEqual,
    Less,
    LessEqual,

    Identifier(&'a str),
    String(&'a str),
    Number(f64),

    And,
    Class,
    Else,
    False,
    Fn,
    Mut,
    For,
    If,
    Unit,
    In,
    Or,
    Return,
    Super,
    This,
    True,
    Let,
    While,
    Eof,
}

impl<'a> Display for Token<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} {} {}", self.token_type, self.lexeme, self.line)
    }
}
