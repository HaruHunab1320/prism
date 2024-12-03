use logos::Logos;
use std::fmt;

#[derive(Logos, Debug, PartialEq, Clone)]
pub enum Token {
    // Keywords
    #[token("conf")]
    Confidence,
    
    #[token("uncertain")]
    Uncertain,
    
    #[token("if")]
    If,
    
    #[token("medium")]
    Medium,
    
    #[token("low")]
    Low,
    
    #[token("in")]
    In,
    
    #[token("context")]
    Context,
    
    #[token("shift")]
    Shift,
    
    #[token("verify")]
    Verify,

    // Operators
    #[token("~>")]
    ConfidenceFlow,
    
    #[token("<~")]
    ReverseConfidenceFlow,
    
    #[token("&&")]
    ConfidenceAnd,
    
    #[token("||")]
    ConfidenceOr,
    
    #[token("=")]
    Assign,

    // Delimiters
    #[token("{")]
    LBrace,
    
    #[token("}")]
    RBrace,
    
    #[token("(")]
    LParen,
    
    #[token(")")]
    RParen,
    
    #[token("[")]
    LBracket,
    
    #[token("]")]
    RBracket,

    // Literals
    #[regex(r"[0-9]+\.[0-9]+", |lex| lex.slice().to_string())]
    Float(String),
    
    #[regex(r"[0-9]+", |lex| lex.slice().to_string())]
    Integer(String),
    
    #[regex(r#""[^"]*""#, |lex| lex.slice().to_string())]
    String(String),
    
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| lex.slice().to_string())]
    Identifier(String),

    // Special
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,
    
    #[regex(r"//[^\n]*", logos::skip)]
    Comment,
    
    Error,
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Float(s) => write!(f, "{}", s),
            Token::Integer(s) => write!(f, "{}", s),
            Token::String(s) => write!(f, "\"{}\"", s),
            Token::Identifier(s) => write!(f, "{}", s),
            _ => write!(f, "{:?}", self),
        }
    }
}

pub struct Lexer<'a> {
    inner: logos::Lexer<'a, Token>,
}

impl<'a> Lexer<'a> {
    pub fn new(input: &'a str) -> Self {
        Self {
            inner: Token::lexer(input),
        }
    }
}

impl<'a> Iterator for Lexer<'a> {
    type Item = Token;

    fn next(&mut self) -> Option<Self::Item> {
        self.inner.next().map(|result| match result {
            Ok(token) => token,
            Err(_) => Token::Error,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let input = "conf x = 0.8";
        let tokens: Vec<Token> = Lexer::new(input).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Confidence,
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Float("0.8".to_string()),
            ]
        );
    }

    #[test]
    fn test_confidence_flow() {
        let input = "x ~> 0.7";
        let tokens: Vec<Token> = Lexer::new(input).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("x".to_string()),
                Token::ConfidenceFlow,
                Token::Float("0.7".to_string()),
            ]
        );
    }

    #[test]
    fn test_comments() {
        let input = "// This is a comment\nconf x = 0.8";
        let tokens: Vec<Token> = Lexer::new(input).collect();
        assert_eq!(
            tokens,
            vec![
                Token::Confidence,
                Token::Identifier("x".to_string()),
                Token::Assign,
                Token::Float("0.8".to_string()),
            ]
        );
    }
} 