use logos::Logos;
use std::error::Error;

#[derive(Debug, Default)]
pub struct LexerExtras {
    pub line: usize,
}

#[derive(Logos, Debug, PartialEq, Clone)]
#[logos(extras = LexerExtras)]
pub enum TokenType {
    // Skip whitespace and comments
    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"\n", |lex| { lex.extras.line += 1; logos::Skip })]
    #[regex(r"//[^\n]*", logos::skip)]

    // Single-character tokens
    #[token("(")]
    LeftParen,
    #[token(")")]
    RightParen,
    #[token("{")]
    LeftBrace,
    #[token("}")]
    RightBrace,
    #[token("[")]
    LeftBracket,
    #[token("]")]
    RightBracket,
    #[token(",")]
    Comma,
    #[token(".")]
    Dot,
    #[token("-")]
    Minus,
    #[token("+")]
    Plus,
    #[token(";")]
    Semicolon,
    #[token("/")]
    Slash,
    #[token("*")]
    Star,

    // One or two character tokens
    #[token("!")]
    Bang,
    #[token("!=")]
    BangEqual,
    #[token("=")]
    Equals,
    #[token("==")]
    EqualEqual,
    #[token(">")]
    Greater,
    #[token(">=")]
    GreaterEqual,
    #[token("<")]
    Less,
    #[token("<=")]
    LessEqual,

    // Literals
    #[regex(r#""[^"]*""#, |lex| Some(lex.slice().to_string()))]
    String(String),
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse().ok())]
    Number(f64),
    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| Some(lex.slice().to_string()))]
    Identifier(String),

    // Keywords
    #[token("and")]
    And,
    #[token("async")]
    Async,
    #[token("break")]
    Break,
    #[token("class")]
    Class,
    #[token("continue")]
    Continue,
    #[token("else")]
    Else,
    #[token("false")]
    False,
    #[token("fn")]
    Fn,
    #[token("for")]
    For,
    #[token("if")]
    If,
    #[token("let")]
    Let,
    #[token("match")]
    Match,
    #[token("null")]
    Null,
    #[token("or")]
    Or,
    #[token("return")]
    Return,
    #[token("super")]
    Super,
    #[token("this")]
    This,
    #[token("true")]
    True,
    #[token("while")]
    While,

    // End of file
    EOF,

    // Error
    #[error]
    Error,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    lexer: logos::Lexer<'a, TokenType>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = TokenType::lexer(source);
        lexer.extras = LexerExtras { line: 1 };
        Self {
            source,
            lexer,
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, Box<dyn Error + Send + Sync>> {
        let mut tokens = Vec::new();

        while let Some(token_type) = self.lexer.next() {
            let lexeme = self.lexer.slice().to_string();
            tokens.push(Token {
                token_type,
                lexeme,
                line: self.lexer.extras.line,
            });
        }

        tokens.push(Token {
            token_type: TokenType::EOF,
            lexeme: String::new(),
            line: self.lexer.extras.line,
        });

        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let source = "let x = 42;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        assert_eq!(tokens.len(), 6); // Including EOF
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("x".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Equals);
        assert_eq!(tokens[3].token_type, TokenType::Number(42.0));
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::EOF);
    }

    #[test]
    fn test_string_literals() {
        let source = r#"let message = "Hello, World!";"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        assert_eq!(tokens.len(), 6); // Including EOF
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("message".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Equals);
        assert_eq!(tokens[3].token_type, TokenType::String("\"Hello, World!\"".to_string()));
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::EOF);
    }

    #[test]
    fn test_number_literals() {
        let source = "let pi = 3.14159;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        assert_eq!(tokens.len(), 6); // Including EOF
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier("pi".to_string()));
        assert_eq!(tokens[2].token_type, TokenType::Equals);
        assert_eq!(tokens[3].token_type, TokenType::Number(3.14159));
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::EOF);
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / == != < <= > >=";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        assert_eq!(tokens.len(), 11); // Including EOF
        assert_eq!(tokens[0].token_type, TokenType::Plus);
        assert_eq!(tokens[1].token_type, TokenType::Minus);
        assert_eq!(tokens[2].token_type, TokenType::Star);
        assert_eq!(tokens[3].token_type, TokenType::Slash);
        assert_eq!(tokens[4].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[5].token_type, TokenType::BangEqual);
        assert_eq!(tokens[6].token_type, TokenType::Less);
        assert_eq!(tokens[7].token_type, TokenType::LessEqual);
        assert_eq!(tokens[8].token_type, TokenType::Greater);
        assert_eq!(tokens[9].token_type, TokenType::GreaterEqual);
        assert_eq!(tokens[10].token_type, TokenType::EOF);
    }

    #[test]
    fn test_keywords() {
        let source = "fn let if else while for return break continue";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        assert_eq!(tokens.len(), 10); // Including EOF
        assert_eq!(tokens[0].token_type, TokenType::Fn);
        assert_eq!(tokens[1].token_type, TokenType::Let);
        assert_eq!(tokens[2].token_type, TokenType::If);
        assert_eq!(tokens[3].token_type, TokenType::Else);
        assert_eq!(tokens[4].token_type, TokenType::While);
        assert_eq!(tokens[5].token_type, TokenType::For);
        assert_eq!(tokens[6].token_type, TokenType::Return);
        assert_eq!(tokens[7].token_type, TokenType::Break);
        assert_eq!(tokens[8].token_type, TokenType::Continue);
        assert_eq!(tokens[9].token_type, TokenType::EOF);
    }

    #[test]
    fn test_line_numbers() {
        let source = "let x = 42;\nlet y = 24;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        assert_eq!(tokens[0].line, 1); // let
        assert_eq!(tokens[4].line, 1); // ;
        assert_eq!(tokens[5].line, 2); // let
        assert_eq!(tokens[9].line, 2); // ;
    }

    #[test]
    fn test_error_handling() {
        let source = "let x = @;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        assert_eq!(tokens[3].token_type, TokenType::Error);
    }
}
