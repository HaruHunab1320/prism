#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    // Single-character tokens
    LeftParen, RightParen, LeftBrace, RightBrace,
    Comma, Dot, Minus, Plus, Semicolon, Slash, Star,

    // One or two character tokens
    Bang, BangEqual,
    Equal, EqualEqual,
    Greater, GreaterEqual,
    Less, LessEqual,
    
    // Confidence operators
    Confidence,      // ~>
    ConfidenceAnd,   // &&
    ConfidenceOr,    // ||
    
    // Context keywords
    In, Context,
    
    // Uncertainty keywords
    UncertainIf, Medium, Low,

    // Literals
    Identifier, String, Number,

    // Keywords
    And, Class, Else, False, Fun, For, If, Nil, Or,
    Return, Super, This, True, Var, While, Let, Fn,
    Async, Await,

    EOF
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
}

impl Token {
    pub fn new(token_type: TokenType, lexeme: String, line: usize) -> Self {
        Self {
            token_type,
            lexeme,
            line,
        }
    }
}

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: &str) -> Self {
        Self {
            source: source.to_string(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>, Box<dyn std::error::Error + Send + Sync>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token::new(
            TokenType::EOF,
            String::new(),
            self.line,
        ));
        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '~' => {
                if self.match_char('>') {
                    self.add_token(TokenType::Confidence)
                } else {
                    return Err(Box::new(crate::error::Error::new("Unexpected character after '~'")));
                }
            },
            '&' => {
                if self.match_char('&') {
                    self.add_token(TokenType::ConfidenceAnd)
                } else {
                    return Err(Box::new(crate::error::Error::new("Unexpected character after '&'")));
                }
            },
            '|' => {
                if self.match_char('|') {
                    self.add_token(TokenType::ConfidenceOr)
                } else {
                    return Err(Box::new(crate::error::Error::new("Unexpected character after '|'")));
                }
            },
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenType::BangEqual)
                } else {
                    self.add_token(TokenType::Bang)
                }
            },
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenType::EqualEqual)
                } else {
                    self.add_token(TokenType::Equal)
                }
            },
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenType::LessEqual)
                } else {
                    self.add_token(TokenType::Less)
                }
            },
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenType::GreaterEqual)
                } else {
                    self.add_token(TokenType::Greater)
                }
            },
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenType::Slash)
                }
            },
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            '"' => self.string()?,
            c if c.is_ascii_digit() => self.number()?,
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier()?,
            _ => return Err(Box::new(crate::error::Error::new(&format!(
                "Unexpected character '{}' at line {}", c, self.line
            )))),
        }
        Ok(())
    }

    fn identifier(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token_type = match text {
            "and" => TokenType::And,
            "class" => TokenType::Class,
            "else" => TokenType::Else,
            "false" => TokenType::False,
            "for" => TokenType::For,
            "fn" => TokenType::Fn,
            "if" => TokenType::If,
            "nil" => TokenType::Nil,
            "or" => TokenType::Or,
            "return" => TokenType::Return,
            "super" => TokenType::Super,
            "this" => TokenType::This,
            "true" => TokenType::True,
            "let" => TokenType::Let,
            "while" => TokenType::While,
            "async" => TokenType::Async,
            "await" => TokenType::Await,
            "in" => TokenType::In,
            "context" => TokenType::Context,
            "uncertain" => TokenType::UncertainIf,
            "medium" => TokenType::Medium,
            "low" => TokenType::Low,
            _ => TokenType::Identifier,
        };

        self.add_token(token_type);
        Ok(())
    }

    fn string(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(Box::new(crate::error::Error::new("Unterminated string.")));
        }

        self.advance(); // The closing ".

        // Trim the surrounding quotes.
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token_with_lexeme(TokenType::String, value);
        Ok(())
    }

    fn number(&mut self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        self.add_token(TokenType::Number);
        Ok(())
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current).unwrap() != expected {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap()
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap()
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap();
        self.current += 1;
        c
    }

    fn add_token(&mut self, token_type: TokenType) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(token_type, text, self.line));
    }

    fn add_token_with_lexeme(&mut self, token_type: TokenType, lexeme: String) {
        self.tokens.push(Token::new(token_type, lexeme, self.line));
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
        
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].lexeme, "x");
        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[3].lexeme, "42");
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::EOF);
    }

    #[test]
    fn test_string_literal() {
        let source = r#"let message = "Hello, World!";"#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].lexeme, "message");
        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[3].token_type, TokenType::String);
        assert_eq!(tokens[3].lexeme, "Hello, World!");
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::EOF);
    }

    #[test]
    fn test_number_literal() {
        let source = "let pi = 3.14159;";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Let);
        assert_eq!(tokens[1].token_type, TokenType::Identifier);
        assert_eq!(tokens[1].lexeme, "pi");
        assert_eq!(tokens[2].token_type, TokenType::Equal);
        assert_eq!(tokens[3].token_type, TokenType::Number);
        assert_eq!(tokens[3].lexeme, "3.14159");
        assert_eq!(tokens[4].token_type, TokenType::Semicolon);
        assert_eq!(tokens[5].token_type, TokenType::EOF);
    }

    #[test]
    fn test_operators() {
        let source = "< <= > >= == != + - * /";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Less);
        assert_eq!(tokens[1].token_type, TokenType::LessEqual);
        assert_eq!(tokens[2].token_type, TokenType::Greater);
        assert_eq!(tokens[3].token_type, TokenType::GreaterEqual);
        assert_eq!(tokens[4].token_type, TokenType::EqualEqual);
        assert_eq!(tokens[5].token_type, TokenType::BangEqual);
        assert_eq!(tokens[6].token_type, TokenType::Plus);
        assert_eq!(tokens[7].token_type, TokenType::Minus);
        assert_eq!(tokens[8].token_type, TokenType::Star);
        assert_eq!(tokens[9].token_type, TokenType::Slash);
        assert_eq!(tokens[10].token_type, TokenType::EOF);
    }

    #[test]
    fn test_keywords() {
        let source = "fn let if else while for return true false nil async await";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens().unwrap();
        
        assert_eq!(tokens[0].token_type, TokenType::Fn);
        assert_eq!(tokens[1].token_type, TokenType::Let);
        assert_eq!(tokens[2].token_type, TokenType::If);
        assert_eq!(tokens[3].token_type, TokenType::Else);
        assert_eq!(tokens[4].token_type, TokenType::While);
        assert_eq!(tokens[5].token_type, TokenType::For);
        assert_eq!(tokens[6].token_type, TokenType::Return);
        assert_eq!(tokens[7].token_type, TokenType::True);
        assert_eq!(tokens[8].token_type, TokenType::False);
        assert_eq!(tokens[9].token_type, TokenType::Nil);
        assert_eq!(tokens[10].token_type, TokenType::Async);
        assert_eq!(tokens[11].token_type, TokenType::Await);
        assert_eq!(tokens[12].token_type, TokenType::EOF);
    }

    #[test]
    fn test_invalid_character() {
        let source = "@";
        let mut lexer = Lexer::new(source);
        let result = lexer.scan_tokens();
        assert!(result.is_err());
    }
}
