use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, Clone, PartialEq)]
pub enum Token {
    #[token("fn")]
    Function,

    #[token("let")]
    Let,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("break")]
    Break,

    #[token("continue")]
    Continue,

    #[token("return")]
    Return,

    #[token("try")]
    Try,

    #[token("catch")]
    Catch,

    #[token("throw")]
    Throw,

    #[token("context")]
    Context,

    #[token("verify")]
    Verify,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("=")]
    Equal,

    #[token("==")]
    EqualEqual,

    #[token("!=")]
    BangEqual,

    #[token("<")]
    Less,

    #[token("<=")]
    LessEqual,

    #[token(">")]
    Greater,

    #[token(">=")]
    GreaterEqual,

    #[token("!")]
    Bang,

    #[token("&&")]
    And,

    #[token("||")]
    Or,

    #[token("(")]
    LParen,

    #[token(")")]
    RParen,

    #[token("{")]
    LBrace,

    #[token("}")]
    RBrace,

    #[token("[")]
    LBracket,

    #[token("]")]
    RBracket,

    #[token(",")]
    Comma,

    #[token(".")]
    Dot,

    #[token(";")]
    Semicolon,

    #[token("~")]
    Tilde,

    #[regex(r#""[^"]*""#, |lex| {
        let slice = lex.slice();
        Some(slice[1..slice.len()-1].to_string())
    })]
    String(String),

    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| {
        lex.slice().parse().ok()
    })]
    Float(f64),

    #[regex(r"true|false", |lex| {
        match lex.slice() {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    })]
    Boolean(bool),

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| {
        Some(lex.slice().to_string())
    })]
    Identifier(String),

    #[regex(r"[ \t\n\r]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Function => write!(f, "fn"),
            Token::Let => write!(f, "let"),
            Token::If => write!(f, "if"),
            Token::Else => write!(f, "else"),
            Token::While => write!(f, "while"),
            Token::For => write!(f, "for"),
            Token::Break => write!(f, "break"),
            Token::Continue => write!(f, "continue"),
            Token::Return => write!(f, "return"),
            Token::Try => write!(f, "try"),
            Token::Catch => write!(f, "catch"),
            Token::Throw => write!(f, "throw"),
            Token::Context => write!(f, "context"),
            Token::Verify => write!(f, "verify"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Star => write!(f, "*"),
            Token::Slash => write!(f, "/"),
            Token::Equal => write!(f, "="),
            Token::EqualEqual => write!(f, "=="),
            Token::BangEqual => write!(f, "!="),
            Token::Less => write!(f, "<"),
            Token::LessEqual => write!(f, "<="),
            Token::Greater => write!(f, ">"),
            Token::GreaterEqual => write!(f, ">="),
            Token::Bang => write!(f, "!"),
            Token::And => write!(f, "&&"),
            Token::Or => write!(f, "||"),
            Token::LParen => write!(f, "("),
            Token::RParen => write!(f, ")"),
            Token::LBrace => write!(f, "{{"),
            Token::RBrace => write!(f, "}}"),
            Token::LBracket => write!(f, "["),
            Token::RBracket => write!(f, "]"),
            Token::Comma => write!(f, ","),
            Token::Dot => write!(f, "."),
            Token::Semicolon => write!(f, ";"),
            Token::Tilde => write!(f, "~"),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Float(n) => write!(f, "{}", n),
            Token::Boolean(b) => write!(f, "{}", b),
            Token::Identifier(name) => write!(f, "{}", name),
            Token::Error => write!(f, "ERROR"),
        }
    }
}

pub struct Lexer<'source> {
    inner: logos::Lexer<'source, Token>,
}

impl<'source> Lexer<'source> {
    pub fn new(source: &'source str) -> Self {
        Self {
            inner: Token::lexer(source),
        }
    }
}

impl<'source> Iterator for Lexer<'source> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().and_then(|result| result.ok())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer() {
        let source = r#"
            fn main() {
                let x = 42.0;
                let s = "hello";
                if true {
                    return x;
                }
            }
        "#;

        let tokens: Vec<_> = Lexer::new(source).collect();
        assert!(tokens.len() > 0);
        assert!(matches!(tokens[0], Token::Function));
        assert!(matches!(tokens[1], Token::Identifier(ref s) if s == "main"));
    }

    #[test]
    fn test_number_lexing() {
        let source = "42.5";
        let tokens: Vec<_> = Lexer::new(source).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Float(n) if (n - 42.5).abs() < f64::EPSILON));
    }

    #[test]
    fn test_string_lexing() {
        let source = r#""hello world""#;
        let tokens: Vec<_> = Lexer::new(source).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::String(ref s) if s == "hello world"));
    }

    #[test]
    fn test_identifier_lexing() {
        let source = "variable_name";
        let tokens: Vec<_> = Lexer::new(source).collect();
        assert_eq!(tokens.len(), 1);
        assert!(matches!(tokens[0], Token::Identifier(ref s) if s == "variable_name"));
    }

    #[test]
    fn test_confidence_flow_lexing() {
        let source = "x ~> 0.9";
        let tokens: Vec<_> = Lexer::new(source).collect();
        assert_eq!(tokens.len(), 4);
        assert!(matches!(tokens[0], Token::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[1], Token::Tilde));
        assert!(matches!(tokens[2], Token::Greater));
        assert!(matches!(tokens[3], Token::Float(n) if (n - 0.9).abs() < f64::EPSILON));
    }

    #[test]
    fn test_context_lexing() {
        let source = r#"context "validation" { let x = 0.8; }"#;
        let tokens: Vec<_> = Lexer::new(source).collect();
        assert!(matches!(tokens[0], Token::Context));
        assert!(matches!(tokens[1], Token::String(ref s) if s == "validation"));
        assert!(matches!(tokens[2], Token::LBrace));
        assert!(matches!(tokens[3], Token::Let));
    }
} 