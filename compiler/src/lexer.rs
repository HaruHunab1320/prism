use std::str::Chars;
use std::iter::Peekable;
use crate::token::{Token, TokenKind};
use crate::error::{PrismError, Result};

pub struct Lexer {
    source: String,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
    chars: String,
    char_pos: usize,
}

impl Lexer {
    pub fn new(source: String) -> Self {
        Self {
            source: source.clone(),
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
            chars: source,
            char_pos: 0,
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
            '-' => {
                if self.match_char('>') {
                    self.add_token(TokenKind::Arrow)
                } else {
                    self.add_token(TokenKind::Minus)
                }
            }
            '+' => self.add_token(TokenKind::Plus),
            ';' => self.add_token(TokenKind::Semicolon),
            '*' => self.add_token(TokenKind::Star),
            '!' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::BangEqual)
                } else {
                    self.add_token(TokenKind::Bang)
                }
            }
            '=' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::EqualEqual)
                } else {
                    self.add_token(TokenKind::Equal)
                }
            }
            '<' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::LessEqual)
                } else {
                    self.add_token(TokenKind::Less)
                }
            }
            '>' => {
                if self.match_char('=') {
                    self.add_token(TokenKind::GreaterEqual)
                } else {
                    self.add_token(TokenKind::Greater)
                }
            }
            '~' => {
                if self.match_char('>') {
                    self.add_token(TokenKind::Confidence)
                } else {
                    return Err(Box::new(PrismError::Parse(
                        format!("Unexpected character '~' at line {}", self.line)
                    )));
                }
            }
            '/' => {
                if self.match_char('/') {
                    // Comment goes until end of line
                    while let Some(c) = self.peek() {
                        if c == '\n' {
                            break;
                        }
                        self.advance();
                    }
                } else {
                    self.add_token(TokenKind::Slash)
                }
            }
            ' ' | '\r' | '\t' => (), // Ignore whitespace
            '\n' => self.line += 1,
            '"' => self.string()?,
            c if c.is_ascii_digit() => self.number()?,
            c if c.is_ascii_alphabetic() || c == '_' => self.identifier()?,
            _ => {
                return Err(Box::new(PrismError::Parse(
                    format!("Unexpected character '{}' at line {}", c, self.line)
                )));
            }
        }
        Ok(())
    }

    fn identifier(&mut self) -> Result<()> {
        while let Some(c) = self.peek() {
            if c.is_ascii_alphanumeric() || c == '_' {
                self.advance();
            } else {
                break;
            }
        }

        let text = &self.source[self.start..self.current];
        let kind = match text {
            "and" => TokenKind::And,
            "class" => TokenKind::Class,
            "else" => TokenKind::Else,
            "false" => TokenKind::False,
            "fun" => TokenKind::Fun,
            "for" => TokenKind::For,
            "if" => TokenKind::If,
            "nil" => TokenKind::Nil,
            "or" => TokenKind::Or,
            "return" => TokenKind::Return,
            "super" => TokenKind::Super,
            "this" => TokenKind::This,
            "true" => TokenKind::True,
            "let" => TokenKind::Let,
            "while" => TokenKind::While,
            "import" => TokenKind::Import,
            "export" => TokenKind::Export,
            "from" => TokenKind::From,
            "as" => TokenKind::As,
            "module" => TokenKind::Module,
            "context" => TokenKind::Context,
            "async" => TokenKind::Async,
            _ => TokenKind::Identifier(text.to_string()),
        };

        self.add_token(kind);
        Ok(())
    }

    fn string(&mut self) -> Result<()> {
        while let Some(c) = self.peek() {
            if c == '"' {
                break;
            }
            if c == '\n' {
                self.line += 1;
            }
            self.advance();
        }

        if self.is_at_end() {
            return Err(Box::new(PrismError::Parse(
                format!("Unterminated string at line {}", self.line)
            )));
        }

        // Consume the closing "
        self.advance();

        // Trim the surrounding quotes
        let value = self.source[self.start + 1..self.current - 1].to_string();
        self.add_token(TokenKind::String(value));
        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        while let Some(c) = self.peek() {
            if c.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }

        // Look for decimal
        if let Some('.') = self.peek() {
            if let Some(next) = self.peek_next() {
                if next.is_ascii_digit() {
                    self.advance(); // Consume the '.'

                    while let Some(c) = self.peek() {
                        if c.is_ascii_digit() {
                            self.advance();
                        } else {
                            break;
                        }
                    }
                }
            }
        }

        let value = self.source[self.start..self.current]
            .parse::<f64>()
            .map_err(|_| {
                PrismError::Parse(format!(
                    "Invalid number '{}' at line {}",
                    &self.source[self.start..self.current],
                    self.line
                ))
            })?;

        self.add_token(TokenKind::Number(value));
        Ok(())
    }

    fn match_char(&mut self, expected: char) -> bool {
        if let Some(c) = self.peek() {
            if c == expected {
                self.advance();
                return true;
            }
        }
        false
    }

    fn advance(&mut self) -> char {
        let c = self.chars[self.char_pos..].chars().next().unwrap_or('\0');
        if c != '\0' {
            self.char_pos += c.len_utf8();
            self.current = self.char_pos;
        }
        c
    }

    fn peek(&self) -> Option<char> {
        self.chars[self.char_pos..].chars().next()
    }

    fn peek_next(&self) -> Option<char> {
        let mut chars = self.chars[self.char_pos..].chars();
        chars.next(); // Skip current
        chars.next()
    }

    fn add_token(&mut self, kind: TokenKind) {
        let text = self.source[self.start..self.current].to_string();
        self.tokens.push(Token::new(kind, text, self.line));
    }

    fn is_at_end(&self) -> bool {
        self.char_pos >= self.chars.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scan_tokens() {
        let mut lexer = Lexer::new("let x = 42;".to_string());
        let tokens = lexer.scan_tokens().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Equal);
        assert_eq!(tokens[3].kind, TokenKind::Number(42.0));
        assert_eq!(tokens[4].kind, TokenKind::Semicolon);
        assert_eq!(tokens[5].kind, TokenKind::EOF);
    }

    #[test]
    fn test_scan_string() {
        let mut lexer = Lexer::new("\"Hello, World!\"".to_string());
        let tokens = lexer.scan_tokens().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::String("Hello, World!".to_string()));
        assert_eq!(tokens[1].kind, TokenKind::EOF);
    }

    #[test]
    fn test_scan_function() {
        let mut lexer = Lexer::new("fun add(a, b)".to_string());
        let tokens = lexer.scan_tokens().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Fun);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("add".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::LeftParen);
        assert_eq!(tokens[3].kind, TokenKind::Identifier("a".to_string()));
        assert_eq!(tokens[4].kind, TokenKind::Comma);
        assert_eq!(tokens[5].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::RightParen);
        assert_eq!(tokens[7].kind, TokenKind::EOF);
    }

    #[test]
    fn test_scan_confidence() {
        let mut lexer = Lexer::new("let x = 42 ~> 0.9;".to_string());
        let tokens = lexer.scan_tokens().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Let);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("x".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::Equal);
        assert_eq!(tokens[3].kind, TokenKind::Number(42.0));
        assert_eq!(tokens[4].kind, TokenKind::Confidence);
        assert_eq!(tokens[5].kind, TokenKind::Number(0.9));
        assert_eq!(tokens[6].kind, TokenKind::Semicolon);
        assert_eq!(tokens[7].kind, TokenKind::EOF);
    }

    #[test]
    fn test_scan_module() {
        let mut lexer = Lexer::new("module math { export fn add(a, b) }".to_string());
        let tokens = lexer.scan_tokens().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Module);
        assert_eq!(tokens[1].kind, TokenKind::Identifier("math".to_string()));
        assert_eq!(tokens[2].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[3].kind, TokenKind::Export);
        assert_eq!(tokens[4].kind, TokenKind::Fun);
        assert_eq!(tokens[5].kind, TokenKind::Identifier("add".to_string()));
        assert_eq!(tokens[6].kind, TokenKind::LeftParen);
        assert_eq!(tokens[7].kind, TokenKind::Identifier("a".to_string()));
        assert_eq!(tokens[8].kind, TokenKind::Comma);
        assert_eq!(tokens[9].kind, TokenKind::Identifier("b".to_string()));
        assert_eq!(tokens[10].kind, TokenKind::RightParen);
        assert_eq!(tokens[11].kind, TokenKind::RightBrace);
        assert_eq!(tokens[12].kind, TokenKind::EOF);
    }

    #[test]
    fn test_scan_import() {
        let mut lexer = Lexer::new("import { add as plus } from \"math\";".to_string());
        let tokens = lexer.scan_tokens().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Import);
        assert_eq!(tokens[1].kind, TokenKind::LeftBrace);
        assert_eq!(tokens[2].kind, TokenKind::Identifier("add".to_string()));
        assert_eq!(tokens[3].kind, TokenKind::As);
        assert_eq!(tokens[4].kind, TokenKind::Identifier("plus".to_string()));
        assert_eq!(tokens[5].kind, TokenKind::RightBrace);
        assert_eq!(tokens[6].kind, TokenKind::From);
        assert_eq!(tokens[7].kind, TokenKind::String("math".to_string()));
        assert_eq!(tokens[8].kind, TokenKind::Semicolon);
        assert_eq!(tokens[9].kind, TokenKind::EOF);
    }
}
