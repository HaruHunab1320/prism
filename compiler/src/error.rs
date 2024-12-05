use crate::lexer::Token;
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    pub token: Token,
    pub message: String,
}

impl ParseError {
    pub fn new(token: &Token, message: &str) -> Self {
        Self {
            token: token.clone(),
            message: message.to_string(),
        }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "[line {}] Error at '{}': {}",
            self.token.line, self.token.lexeme, self.message
        )
    }
}

impl Error for ParseError {}

impl From<String> for ParseError {
    fn from(message: String) -> Self {
        Self {
            token: Token {
                token_type: crate::lexer::TokenType::Error,
                lexeme: String::new(),
                line: 0,
            },
            message,
        }
    }
}

impl From<ParseError> for String {
    fn from(error: ParseError) -> Self {
        error.message
    }
}

#[derive(Debug)]
pub struct RuntimeError {
    pub message: String,
}

impl RuntimeError {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Runtime Error: {}", self.message)
    }
}

impl Error for RuntimeError {}

impl From<String> for RuntimeError {
    fn from(message: String) -> Self {
        Self { message }
    }
}

impl From<RuntimeError> for String {
    fn from(error: RuntimeError) -> Self {
        error.message
    }
}
