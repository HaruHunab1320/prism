use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenKind {
    // Single-character tokens
    LeftParen, RightParen,
    LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus,
    Semicolon, Slash, Star,

    // One or two character tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    Arrow,      // =>
    Confidence, // ~>

    // Literals
    Identifier(String),
    String(String),
    Number(f64),

    // Keywords
    And, Class, Else, False,
    Fun, For, If, Nil, Or,
    Return, Super, This, True,
    Let, While, Break, Continue,
    Import, Export, From, Module,
    In, Context, As, Async,

    EOF,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Token {
    pub kind: TokenKind,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(kind: TokenKind, lexeme: String, line: usize) -> Self {
        Self {
            kind,
            lexeme,
            line,
        }
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?} {}", self.kind, self.lexeme)
    }
} 