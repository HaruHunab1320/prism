use logos::Logos;
use std::fmt;
use std::error::Error;

#[derive(Debug)]
pub struct LexerError(String);

impl fmt::Display for LexerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Lexer error: {}", self.0)
    }
}

impl Error for LexerError {}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(extras = LexerExtras)]
pub enum TokenType {
    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"\n", |lex| { lex.extras.line += 1; logos::Skip })]
    #[error]
    Error,

    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse().ok())]
    Number(f64),

    #[regex(r#""[^"]*""#, |lex| Some(lex.slice()[1..lex.slice().len()-1].to_string()))]
    String(String),

    #[regex(r#""[^"]*"#, |_| None)]
    UnterminatedString,

    #[token("true")]
    True,

    #[token("false")]
    False,

    #[token("null")]
    Null,

    #[token("let")]
    Let,

    #[token("fn")]
    Fn,

    #[token("if")]
    If,

    #[token("else")]
    Else,

    #[token("match")]
    Match,

    #[token("return")]
    Return,

    #[token("while")]
    While,

    #[token("for")]
    For,

    #[token("in")]
    In,

    #[token("try")]
    Try,

    #[token("with")]
    With,

    #[token("confidence")]
    Confidence,

    #[token("verify")]
    Verify,

    #[token("pattern")]
    Pattern,

    #[token("async")]
    Async,

    #[token("+")]
    Plus,

    #[token("-")]
    Minus,

    #[token("*")]
    Star,

    #[token("/")]
    Slash,

    #[token("=")]
    Equals,

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

    #[token(";")]
    Semicolon,

    #[token(":")]
    Colon,

    #[token("=>")]
    Arrow,

    #[regex(r"[a-zA-Z_][a-zA-Z0-9_]*", |lex| Some(lex.slice().to_string()))]
    Identifier(String),

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

#[derive(Debug, Default)]
pub struct LexerExtras {
    pub line: usize,
}

pub struct Lexer<'a> {
    source: &'a str,
    lexer: logos::Lexer<'a, TokenType>,
    line: usize,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = TokenType::lexer(source);
        lexer.extras = LexerExtras { line: 1 };
        Self {
            source,
            lexer,
            line: 1,
        }
    }

    pub fn lex(&mut self) -> Result<Vec<Token>, Box<dyn Error + Send + Sync>> {
        let mut tokens = Vec::new();
        
        while let Some(token_type) = self.lexer.next() {
            match token_type {
                TokenType::Error => {
                    return Err(Box::new(LexerError(format!(
                        "Invalid character at line {}", 
                        self.lexer.extras.line
                    ))));
                }
                TokenType::UnterminatedString => {
                    return Err(Box::new(LexerError(format!(
                        "Unterminated string at line {}", 
                        self.lexer.extras.line
                    ))));
                }
                token_type => {
                    tokens.push(Token {
                        token_type,
                        lexeme: self.lexer.slice().to_string(),
                        line: self.lexer.extras.line,
                    });
                }
            }
        }

        // Add EOF token
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
        
        assert_eq!(tokens.len(), 6); // let, x, =, 42, ;, EOF
        
        assert!(matches!(tokens[0].token_type, TokenType::Let));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(ref s) if s == "x"));
        assert!(matches!(tokens[2].token_type, TokenType::Equals));
        assert!(matches!(tokens[3].token_type, TokenType::Number(n) if n == 42.0));
        assert!(matches!(tokens[4].token_type, TokenType::Semicolon));
        assert!(matches!(tokens[5].token_type, TokenType::EOF));
    }

    #[test]
    fn test_string_literals() {
        let source = r#"let msg = "hello world";"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        
        assert_eq!(tokens.len(), 6); // let, msg, =, "hello world", ;, EOF
        assert!(matches!(tokens[3].token_type, TokenType::String(ref s) if s == "hello world"));
    }

    #[test]
    fn test_number_literals() {
        let source = "42 3.14 0.1 100.0";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::Number(n) if n == 42.0));
        assert!(matches!(tokens[1].token_type, TokenType::Number(n) if (n - 3.14).abs() < f64::EPSILON));
        assert!(matches!(tokens[2].token_type, TokenType::Number(n) if n == 0.1));
        assert!(matches!(tokens[3].token_type, TokenType::Number(n) if n == 100.0));
    }

    #[test]
    fn test_operators() {
        let source = "+ - * / == != < <= > >= && ||";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::Plus));
        assert!(matches!(tokens[1].token_type, TokenType::Minus));
        assert!(matches!(tokens[2].token_type, TokenType::Star));
        assert!(matches!(tokens[3].token_type, TokenType::Slash));
        assert!(matches!(tokens[4].token_type, TokenType::EqualEqual));
        assert!(matches!(tokens[5].token_type, TokenType::BangEqual));
        assert!(matches!(tokens[6].token_type, TokenType::Less));
        assert!(matches!(tokens[7].token_type, TokenType::LessEqual));
        assert!(matches!(tokens[8].token_type, TokenType::Greater));
        assert!(matches!(tokens[9].token_type, TokenType::GreaterEqual));
        assert!(matches!(tokens[10].token_type, TokenType::And));
        assert!(matches!(tokens[11].token_type, TokenType::Or));
    }

    #[test]
    fn test_keywords() {
        let source = "let fn if else match try verify return true false null while for async";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::Let));
        assert!(matches!(tokens[1].token_type, TokenType::Fn));
        assert!(matches!(tokens[2].token_type, TokenType::If));
        assert!(matches!(tokens[3].token_type, TokenType::Else));
        assert!(matches!(tokens[4].token_type, TokenType::Match));
        assert!(matches!(tokens[5].token_type, TokenType::Try));
        assert!(matches!(tokens[6].token_type, TokenType::Verify));
        assert!(matches!(tokens[7].token_type, TokenType::Return));
        assert!(matches!(tokens[8].token_type, TokenType::True));
        assert!(matches!(tokens[9].token_type, TokenType::False));
        assert!(matches!(tokens[10].token_type, TokenType::Null));
        assert!(matches!(tokens[11].token_type, TokenType::While));
        assert!(matches!(tokens[12].token_type, TokenType::For));
        assert!(matches!(tokens[13].token_type, TokenType::Async));
    }

    #[test]
    fn test_line_numbers() {
        let source = "let x = 42;\nlet y = 10;\nif true {\n    return 1;\n}";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        
        // First line
        assert_eq!(tokens[0].line, 1); // let
        assert_eq!(tokens[4].line, 1); // ;
        
        // Second line
        assert_eq!(tokens[5].line, 2); // let
        assert_eq!(tokens[9].line, 2); // ;
        
        // Third and fourth lines
        assert_eq!(tokens[10].line, 3); // if
        assert_eq!(tokens[14].line, 4); // return
    }

    #[test]
    fn test_error_handling() {
        // Invalid character
        let source = "let x = @;";
        let mut lexer = Lexer::new(source);
        assert!(lexer.lex().is_err());

        // Unterminated string
        let source = r#"let msg = "hello"#;
        let mut lexer = Lexer::new(source);
        assert!(lexer.lex().is_err());
    }
}
