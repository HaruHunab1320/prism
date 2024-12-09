use crate::token::{Token, TokenKind};
use crate::error::{PrismError, Result};

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens.push(Token::new(
            TokenKind::EOF,
            String::new(),
            self.line,
        ));

        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<()> {
        let c = self.advance();
        match c {
            '(' => self.add_token(TokenKind::LeftParen),
            ')' => self.add_token(TokenKind::RightParen),
            '{' => self.add_token(TokenKind::LeftBrace),
            '}' => self.add_token(TokenKind::RightBrace),
            ',' => self.add_token(TokenKind::Comma),
            '.' => self.add_token(TokenKind::Dot),
            '-' => self.add_token(TokenKind::Minus),
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),
            '!' => {
                let token = if self.match_char('=') {
                    TokenKind::BangEqual
                } else {
                    TokenKind::Bang
                };
                self.add_token(token);
            }
            '=' => {
                let token = if self.match_char('=') {
                    TokenKind::EqualEqual
                } else if self.match_char('>') {
                    TokenKind::Arrow
                } else {
                    TokenKind::Equal
                };
                self.add_token(token);
            }
            '<' => {
                let token = if self.match_char('=') {
                    TokenKind::LessEqual
                } else {
                    TokenKind::Less
                };
                self.add_token(token);
            }
            '>' => {
                let token = if self.match_char('=') {
                    TokenKind::GreaterEqual
                } else {
                    TokenKind::Greater
                };
                self.add_token(token);
            }
            '~' => {
                if self.match_char('>') {
                    self.add_token(TokenKind::Confidence);
                } else {
                    return Err(PrismError::ParseError(
                        format!("Unexpected character '~' at line {}", self.line)
                    ));
                }
            }
            '"' => self.string()?,
            '/' => {
                if self.match_char('/') {
                    while self.peek() != '\n' && !self.is_at_end() {
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash);
                }
            }
            ' ' | '\r' | '\t' => (),
            '\n' => self.line += 1,
            c if c.is_ascii_digit() => self.number()?,
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier()?,
            _ => {
                return Err(PrismError::ParseError(
                    format!("Unexpected character '{}' at line {}", c, self.line)
                ));
            }
        }
        Ok(())
    }

    fn identifier(&mut self) -> Result<()> {
        while self.peek().is_ascii_alphanumeric() || self.peek() == '_' {
            self.advance();
        }

        let text = &self.source[self.start..self.current];
        let token = match text {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "for" => TokenKind::For,
            "fn" => TokenKind::Fun,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "let" => TokenKind::Let,
            "while" => TokenKind::While,
            "break" => TokenKind::Break,
            "continue" => TokenKind::Continue,
            "import" => TokenKind::Import,
            "export" => TokenKind::Export,
            "from" => TokenKind::From,
            "module" => TokenKind::Module,
            "in" => TokenKind::In,
            "context" => TokenKind::Context,
            "as" => TokenKind::As,
            "async" => TokenKind::Async,
            _ => TokenKind::Identifier(text.to_string()),
        };

        self.add_token(token);
        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        while self.peek().is_ascii_digit() {
            self.advance();
        }

        if self.peek() == '.' && self.peek_next().is_ascii_digit() {
            self.advance();

            while self.peek().is_ascii_digit() {
                self.advance();
            }
        }

        let value = self.source[self.start..self.current]
            .parse::<f64>()
            .map_err(|_| {
                PrismError::ParseError(format!(
                    "Invalid number at line {}",
                    self.line
                ))
            })?;

        self.add_token(TokenKind::Number(value));
        Ok(())
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != '"' && !self.is_at_end() {
            if self.peek() == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(PrismError::ParseError(
                format!("Unterminated string at line {}", self.line)
            ));
        }

        self.advance();

        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenKind::String(value));
        Ok(())
    }

    fn match_char(&mut self, expected: char) -> bool {
        if self.is_at_end() {
            return false;
        }
        if self.source.chars().nth(self.current) != Some(expected) {
            return false;
        }

        self.current += 1;
        true
    }

    fn peek(&self) -> char {
        if self.is_at_end() {
            '\0'
        } else {
            self.source.chars().nth(self.current).unwrap_or('\0')
        }
    }

    fn peek_next(&self) -> char {
        if self.current + 1 >= self.source.len() {
            '\0'
        } else {
            self.source.chars().nth(self.current + 1).unwrap_or('\0')
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn advance(&mut self) -> char {
        let c = self.source.chars().nth(self.current).unwrap_or('\0');
        self.current += 1;
        c
    }

    fn add_token(&mut self, kind: TokenKind) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(kind, text, self.line));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_tokens() -> Result<()> {
        let source = "let x = 42;".to_string();
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens()?;

        assert_eq!(tokens.len(), 6); // let, x, =, 42, ;, EOF
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Equal);
        assert_eq!(tokens[3].kind, TokenKind::Number(42.0));
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
        assert_eq!(tokens[5].kind, TokenKind::EOF);

        Ok(())
    }

    #[test]
    fn test_scan_string() -> Result<()> {
        let source = r#"let x = "hello";"#.to_string();
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens()?;

        assert_eq!(tokens.len(), 6); // let, x, =, "hello", ;, EOF
        assert_eq!(tokens[3].kind, TokenKind::String("hello".to_string()));

        Ok(())
    }

    #[test]
    fn test_scan_function() -> Result<()> {
        let source = "fn add(a, b) { return a + b; }".to_string();
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens()?;

        assert_eq!(tokens[0].kind, TokenKind::Fun);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("add".to_string()));

        Ok(())
    }

    #[test]
    fn test_scan_module() -> Result<()> {
        let source = r#"
            module test {
                fn add(a, b) {
                    return a + b;
                }
            }
        "#.to_string();
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens()?;

        // Skip whitespace tokens
        let mut i = 0;
        while i < tokens.len() && tokens[i].kind == TokenKind::Identifier("".to_string()) {
            i += 1;
        }

        assert_eq!(tokens[i].kind, TokenKind::Module);
        assert_eq!(tokens[i + 1].kind, TokenKind::Identifier("test".to_string()));

        Ok(())
    }

    #[test]
    fn test_scan_import() -> Result<()> {
        let source = r#"import { add } from "math";"#.to_string();
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens()?;

        assert_eq!(tokens[0].kind, TokenKind::Import);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("add".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::From);
        assert_eq!(tokens[5].kind, TokenKind::String("math".to_string()));

        Ok(())
    }

    #[test]
    fn test_scan_confidence() -> Result<()> {
        let source = "let x = 42 ~> 0.9;".to_string();
        let mut lexer = Lexer::new(source);
        let tokens = lexer.scan_tokens()?;

        assert_eq!(tokens[4].kind, TokenKind::Confidence);
        assert_eq!(tokens[5].kind, TokenKind::Number(0.9));

        Ok(())
    }
}
