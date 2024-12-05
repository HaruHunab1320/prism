use logos::Logos;

#[derive(Default)]
pub struct LexerState {
    pub line: usize,
}

#[derive(Logos, Debug, Clone, PartialEq)]
#[logos(extras = LexerState)]
pub enum TokenType {
    // Skip whitespace and comments
    #[regex(r"[ \t\f]+", logos::skip)]
    #[regex(r"//[^\n]*", logos::skip)]
    #[regex(r"\n", |lex| { lex.extras.line += 1; logos::skip(lex) })]
    Whitespace,

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
    Equal,
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
    #[regex("[a-zA-Z_][a-zA-Z0-9_]*", |lex| Some(lex.slice().to_string()))]
    Identifier(String),
    #[regex(r#""[^"]*""#, |lex| Some(lex.slice()[1..lex.slice().len()-1].to_string()))]
    String(String),
    #[regex(r"[0-9]+(\.[0-9]+)?", |lex| lex.slice().parse().ok())]
    Number(f64),

    // Keywords
    #[token("let")]
    Let,
    #[token("fn")]
    Function,
    #[token("if")]
    If,
    #[token("else")]
    Else,
    #[token("return")]
    Return,
    #[token("while")]
    While,
    #[token("true")]
    True,
    #[token("false")]
    False,
    #[token("null")]
    Null,
    #[token("and")]
    And,
    #[token("or")]
    Or,
    #[token("for")]
    For,
    #[token("async")]
    Async,
    #[token("await")]
    Await,
    #[token("match")]
    Match,
    #[token("try")]
    Try,
    #[token("verify")]
    Verify,
    #[token("class")]
    Class,
    #[token("super")]
    Super,
    #[token("this")]
    This,

    #[error]
    Error,

    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

pub struct Lexer<'a> {
    lexer: logos::Lexer<'a, TokenType>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut lexer = TokenType::lexer(source);
        lexer.extras = LexerState { line: 1 };
        Self { lexer }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        while let Some(token_type) = self.lexer.next() {
            match token_type {
                TokenType::Error => {
                    return Err(format!("Unexpected character at line {}", self.lexer.extras.line));
                }
                TokenType::Whitespace => continue,
                _ => {
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
        let tokens = lexer.scan_tokens().unwrap();
        
        assert_eq!(tokens.len(), 6); // let, x, =, 42, ;, EOF
        
        assert!(matches!(tokens[0].token_type, TokenType::Let));
        assert!(matches!(tokens[1].token_type, TokenType::Identifier(_)));
        assert!(matches!(tokens[2].token_type, TokenType::Equal));
        assert!(matches!(tokens[3].token_type, TokenType::Number(_)));
        assert!(matches!(tokens[4].token_type, TokenType::Semicolon));
        assert!(matches!(tokens[5].token_type, TokenType::EOF));
    }

    #[test]
    fn test_string_literal() {
        let source = r#"let msg = "hello";"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().unwrap();
        
        assert_eq!(tokens.len(), 6); // let, msg, =, "hello", ;, EOF
        if let TokenType::String(s) = &tokens[3].token_type {
            assert_eq!(s, "hello");
        } else {
            panic!("Expected string token");
        }
    }

    #[test]
    fn test_line_numbers() {
        let source = "let x = 42;\nlet y = 10;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().unwrap();
        
        assert_eq!(tokens[0].line, 1); // First line tokens
        assert_eq!(tokens[4].line, 1); // Last token of first line
        assert_eq!(tokens[5].line, 2); // First token of second line
        assert_eq!(tokens[9].line, 2); // Last token of second line
        assert_eq!(tokens[10].line, 2); // EOF token
    }

    #[test]
    fn test_keywords() {
        let source = "if true { return false; }";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().unwrap();
        
        assert!(matches!(tokens[0].token_type, TokenType::If));
        assert!(matches!(tokens[1].token_type, TokenType::True));
        assert!(matches!(tokens[2].token_type, TokenType::LeftBrace));
        assert!(matches!(tokens[3].token_type, TokenType::Return));
        assert!(matches!(tokens[4].token_type, TokenType::False));
        assert!(matches!(tokens[5].token_type, TokenType::Semicolon));
        assert!(matches!(tokens[6].token_type, TokenType::RightBrace));
    }

    #[test]
    fn test_error_handling() {
        let source = "let x = @;";
        let mut lexer = Lexer::new(source);
        assert!(lexer.scan_tokens().is_err());
    }
}
