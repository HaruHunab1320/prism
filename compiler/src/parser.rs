use crate::ast::{Expr, Stmt};
use crate::lexer::Token;
use std::iter::Peekable;
use std::fmt;
use std::error::Error;
use miette::{Diagnostic, SourceSpan};
use thiserror::Error;

#[derive(Error, Debug, Diagnostic)]
#[error("Parse error: {kind}")]
pub struct ParseError {
    #[source_code]
    pub source_code: String,
    #[label("here")]
    pub span: SourceSpan,
    pub kind: ParseErrorKind,
}

#[derive(Error, Debug)]
pub enum ParseErrorKind {
    #[error("unexpected token: expected {expected}, found {found}")]
    UnexpectedToken {
        expected: String,
        found: String,
    },
    #[error("unexpected end of file: expected {expected}")]
    UnexpectedEof {
        expected: String,
    },
    #[error("invalid expression: {0}")]
    InvalidExpression(String),
}

impl ParseError {
    pub fn unexpected_token(expected: &str, found: String, position: usize) -> Self {
        Self {
            source_code: String::new(),
            span: (position, 1).into(),
            kind: ParseErrorKind::UnexpectedToken {
                expected: expected.to_string(),
                found,
            },
        }
    }

    pub fn unexpected_eof(expected: &str, position: usize) -> Self {
        Self {
            source_code: String::new(),
            span: (position, 1).into(),
            kind: ParseErrorKind::UnexpectedEof {
                expected: expected.to_string(),
            },
        }
    }

    pub fn invalid_expression(message: String, position: usize) -> Self {
        Self {
            source_code: String::new(),
            span: (position, 1).into(),
            kind: ParseErrorKind::InvalidExpression(message),
        }
    }

    pub fn location(&self) -> usize {
        self.span.offset()
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source_code = source;
        self
    }
}

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    tokens: Peekable<I>,
    position: usize,
    source: String,
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
            position: 0,
            source: String::new(),
        }
    }

    pub fn with_source(mut self, source: String) -> Self {
        self.source = source;
        self
    }

    pub fn position(&self) -> usize {
        self.position
    }

    pub fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.tokens.peek() {
            Some(Token::Confidence) => self.parse_confidence_declaration(),
            Some(Token::Uncertain) => self.parse_uncertain_if(),
            Some(Token::In) => self.parse_context_block(),
            Some(Token::Verify) => self.parse_verification_block(),
            Some(Token::Function) => self.parse_function_definition(),
            Some(_) => self.parse_expression().map(Stmt::Expression),
            None => Err(ParseError::unexpected_eof("statement", self.position))
                .map_err(|e| e.with_source(self.source.clone())),
        }
    }

    fn expect_token(&mut self, expected: Token) -> Result<(), ParseError> {
        match self.tokens.next() {
            Some(token) if token == expected => {
                self.position += 1;
                Ok(())
            }
            Some(token) => Err(ParseError::unexpected_token(
                &format!("{:?}", expected),
                format!("{:?}", token),
                self.position,
            ).with_source(self.source.clone())),
            None => Err(ParseError::unexpected_eof(
                &format!("{:?}", expected),
                self.position,
            ).with_source(self.source.clone())),
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while self.tokens.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        match self.tokens.next() {
            Some(Token::Float(s)) => {
                self.position += 1;
                s.parse::<f64>()
                    .map(Expr::Float)
                    .map_err(|_| ParseError::invalid_expression(format!("Invalid float: {}", s), self.position))
            }
            Some(Token::Integer(s)) => {
                self.position += 1;
                s.parse::<i64>()
                    .map(Expr::Integer)
                    .map_err(|_| ParseError::invalid_expression(format!("Invalid integer: {}", s), self.position))
            }
            Some(Token::String(s)) => {
                self.position += 1;
                Ok(Expr::String(s))
            }
            Some(Token::Identifier(s)) => {
                self.position += 1;
                Ok(Expr::Identifier(s))
            }
            Some(token) => Err(ParseError::unexpected_token(
                "expression",
                format!("{:?}", token),
                self.position,
            )),
            None => Err(ParseError::unexpected_eof(
                "expression",
                self.position,
            )),
        }
    }

    fn parse_confidence_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(Token::Confidence)?;
        let name = match self.tokens.next() {
            Some(Token::Identifier(name)) => {
                self.position += 1;
                name
            }
            Some(token) => return Err(ParseError::unexpected_token(
                "identifier",
                format!("{:?}", token),
                self.position,
            )),
            None => return Err(ParseError::unexpected_eof(
                "identifier",
                self.position,
            )),
        };

        self.expect_token(Token::Assign)?;
        let value = self.parse_expression()?;

        Ok(Stmt::Declaration { name, value })
    }

    fn parse_uncertain_if(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(Token::Uncertain)?;
        self.expect_token(Token::If)?;
        self.expect_token(Token::LParen)?;

        let condition = self.parse_expression()?;

        self.expect_token(Token::RParen)?;

        let mut high_confidence = Vec::new();
        self.expect_token(Token::LBrace)?;
        while let Some(token) = self.tokens.peek() {
            if *token == Token::RBrace {
                self.tokens.next();
                break;
            }
            high_confidence.push(self.parse_statement()?);
        }

        let medium_confidence = if let Some(Token::Medium) = self.tokens.peek() {
            self.tokens.next();
            let mut stmts = Vec::new();
            self.expect_token(Token::LBrace)?;
            while let Some(token) = self.tokens.peek() {
                if *token == Token::RBrace {
                    self.tokens.next();
                    break;
                }
                stmts.push(self.parse_statement()?);
            }
            Some(stmts)
        } else {
            None
        };

        let low_confidence = if let Some(Token::Low) = self.tokens.peek() {
            self.tokens.next();
            let mut stmts = Vec::new();
            self.expect_token(Token::LBrace)?;
            while let Some(token) = self.tokens.peek() {
                if *token == Token::RBrace {
                    self.tokens.next();
                    break;
                }
                stmts.push(self.parse_statement()?);
            }
            Some(stmts)
        } else {
            None
        };

        Ok(Stmt::Expression(Expr::UncertainIf {
            condition: Box::new(condition),
            high_confidence,
            medium_confidence,
            low_confidence,
        }))
    }

    fn parse_context_block(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(Token::In)?;
        self.expect_token(Token::Context)?;

        let context_name = match self.tokens.next() {
            Some(Token::Identifier(name)) => {
                self.position += 1;
                name
            }
            Some(token) => return Err(ParseError::unexpected_token(
                "identifier",
                format!("{:?}", token),
                self.position,
            )),
            None => return Err(ParseError::unexpected_eof(
                "identifier",
                self.position,
            )),
        };

        let mut body = Vec::new();
        self.expect_token(Token::LBrace)?;
        while let Some(token) = self.tokens.peek() {
            if *token == Token::RBrace {
                self.tokens.next();
                break;
            }
            body.push(self.parse_statement()?);
        }

        Ok(Stmt::Expression(Expr::ContextBlock {
            context_name,
            body,
        }))
    }

    fn parse_verification_block(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(Token::Verify)?;
        self.expect_token(Token::Identifier("against".into()))?;
        self.expect_token(Token::Identifier("sources".into()))?;

        let mut sources = Vec::new();
        self.expect_token(Token::LBracket)?;
        while let Some(token) = self.tokens.peek() {
            if *token == Token::RBracket {
                self.tokens.next();
                break;
            }
            match self.tokens.next() {
                Some(Token::String(s)) => {
                    self.position += 1;
                    sources.push(s);
                }
                Some(token) => return Err(ParseError::unexpected_token(
                    "string",
                    format!("{:?}", token),
                    self.position,
                )),
                None => return Err(ParseError::unexpected_eof(
                    "string or ]",
                    self.position,
                )),
            }
        }

        let threshold = match self.tokens.next() {
            Some(Token::Float(s)) => {
                self.position += 1;
                s.parse::<f64>().map_err(|_| ParseError::invalid_expression(
                    format!("Invalid float: {}", s),
                    self.position,
                ))?
            }
            Some(token) => return Err(ParseError::unexpected_token(
                "float",
                format!("{:?}", token),
                self.position,
            )),
            None => return Err(ParseError::unexpected_eof(
                "float",
                self.position,
            )),
        };

        let mut body = Vec::new();
        self.expect_token(Token::LBrace)?;
        while let Some(token) = self.tokens.peek() {
            if *token == Token::RBrace {
                self.tokens.next();
                break;
            }
            body.push(self.parse_statement()?);
        }

        Ok(Stmt::Expression(Expr::Verify {
            sources,
            threshold,
            body,
        }))
    }

    fn parse_function_definition(&mut self) -> Result<Stmt, ParseError> {
        self.expect_token(Token::Function)?;
        
        let name = if let Some(Token::Identifier(name)) = &self.tokens.peek() {
            name.clone()
        } else {
            return Err(ParseError::new("Expected function name"));
        };
        self.tokens.next();
        
        // Parse parameters
        self.expect_token(Token::LParen)?;
        let mut parameters = Vec::new();
        
        while self.tokens.peek() != Some(Token::RParen) {
            if let Some(Token::Identifier(param)) = &self.tokens.peek() {
                parameters.push(param.clone());
                self.tokens.next();
                
                match self.tokens.peek() {
                    Some(Token::Comma) => {
                        self.tokens.next();
                    }
                    Some(Token::RParen) => {}
                    _ => return Err(ParseError::new("Expected ',' or ')' after parameter")),
                }
            } else {
                return Err(ParseError::new("Expected parameter name"));
            }
        }
        self.tokens.next(); // consume ')'
        
        // Parse optional confidence level
        let confidence_level = if self.tokens.peek() == Some(Token::ConfidenceFlow) {
            self.tokens.next();
            if let Some(Token::Float(conf)) = self.tokens.peek() {
                self.tokens.next();
                Some(conf)
            } else {
                return Err(ParseError::new("Expected confidence value after '~'"));
            }
        } else {
            None
        };
        
        // Parse function body
        self.expect_token(Token::LBrace)?;
        let mut body = Vec::new();
        
        while self.tokens.peek() != Some(Token::RBrace) {
            body.push(self.parse_statement()?);
        }
        self.tokens.next(); // consume '}'
        
        Ok(Stmt::FunctionDefinition {
            name,
            parameters,
            body,
            confidence_level,
        })
    }
} 