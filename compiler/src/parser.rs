use crate::ast::{Expr, Stmt, UnaryOp, BinaryOp, Value};
use crate::lexer::{Token, TokenType};
use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct ParseError {
    token: Token,
    message: String,
}

impl ParseError {
    fn new(token: Token, message: String) -> Self {
        Self { token, message }
    }
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Parse error at line {}: {}",
            self.token.line,
            self.message
        )
    }
}

impl Error for ParseError {}

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Box<dyn Error + Send + Sync>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, Box<dyn Error + Send + Sync>> {
        if self.match_token(&TokenType::Let) {
            self.let_declaration()
        } else if self.match_token(&TokenType::Fn) {
            self.function_declaration()
        } else {
            self.statement()
        }
    }

    fn let_declaration(&mut self) -> Result<Stmt, Box<dyn Error + Send + Sync>> {
        let name = if let TokenType::Identifier(name) = self.consume_identifier()?.token_type {
            name
        } else {
            return Err(Box::new(ParseError::new(
                self.previous(),
                "Expected variable name".to_string(),
            )));
        };

        let initializer = if self.match_token(&TokenType::Equals) {
            self.expression()?
        } else {
            return Err(Box::new(ParseError::new(
                self.previous(),
                "Expected '=' after variable name".to_string(),
            )));
        };

        self.consume(&TokenType::Semicolon, "Expected ';' after variable declaration")?;
        Ok(Stmt::Let(name, initializer))
    }

    fn function_declaration(&mut self) -> Result<Stmt, Box<dyn Error + Send + Sync>> {
        let name = if let TokenType::Identifier(name) = self.consume_identifier()?.token_type {
            name
        } else {
            return Err(Box::new(ParseError::new(
                self.previous(),
                "Expected function name".to_string(),
            )));
        };

        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;
        
        let mut parameters = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if parameters.len() >= 255 {
                    return Err(Box::new(ParseError::new(
                        self.peek(),
                        "Cannot have more than 255 parameters".to_string(),
                    )));
                }

                if let TokenType::Identifier(param) = self.consume_identifier()?.token_type {
                    parameters.push(param);
                } else {
                    return Err(Box::new(ParseError::new(
                        self.previous(),
                        "Expected parameter name".to_string(),
                    )));
                }

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;
        self.consume(&TokenType::LeftBrace, "Expected '{' before function body")?;

        let body = Box::new(self.block()?);
        Ok(Stmt::Function {
            name,
            params: parameters,
            body,
            is_async: false,
        })
    }

    fn statement(&mut self) -> Result<Stmt, Box<dyn Error + Send + Sync>> {
        if self.match_token(&TokenType::If) {
            self.if_statement()
        } else if self.match_token(&TokenType::LeftBrace) {
            Ok(Stmt::Block(self.block_statements()?))
        } else if self.match_token(&TokenType::Return) {
            self.return_statement()
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, Box<dyn Error + Send + Sync>> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after if condition")?;

        let then_branch = if self.match_token(&TokenType::LeftBrace) {
            Box::new(Stmt::Block(self.block_statements()?))
        } else {
            Box::new(self.statement()?)
        };

        let else_branch = if self.match_token(&TokenType::Else) {
            if self.match_token(&TokenType::LeftBrace) {
                Some(Box::new(Stmt::Block(self.block_statements()?)))
            } else {
                Some(Box::new(self.statement()?))
            }
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Stmt, Box<dyn Error + Send + Sync>> {
        let statements = self.block_statements()?;
        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        Ok(Stmt::Block(statements))
    }

    fn block_statements(&mut self) -> Result<Vec<Stmt>, Box<dyn Error + Send + Sync>> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        Ok(statements)
    }

    fn return_statement(&mut self) -> Result<Stmt, Box<dyn Error + Send + Sync>> {
        let value = Box::new(self.expression()?);
        self.consume(&TokenType::Semicolon, "Expected ';' after return value")?;
        Ok(Stmt::Return(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Box<dyn Error + Send + Sync>> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        let expr = self.or()?;

        if self.match_token(&TokenType::Equals) {
            let equals = self.previous();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Binary {
                    left: Box::new(Expr::Variable(name)),
                    operator: BinaryOp::Equal,
                    right: Box::new(value),
                });
            }

            return Err(Box::new(ParseError::new(
                equals,
                "Invalid assignment target".to_string(),
            )));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        let mut expr = self.and()?;

        while self.match_token(&TokenType::Or) {
            let operator = BinaryOp::Or;
            let right = self.and()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        let mut expr = self.equality()?;

        while self.match_token(&TokenType::And) {
            let operator = BinaryOp::And;
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        let mut expr = self.comparison()?;

        while self.match_any(&[&TokenType::BangEqual, &TokenType::EqualEqual]) {
            let operator = match self.previous().token_type {
                TokenType::BangEqual => BinaryOp::NotEqual,
                TokenType::EqualEqual => BinaryOp::Equal,
                _ => unreachable!(),
            };
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        let mut expr = self.term()?;

        while self.match_any(&[
            &TokenType::Greater,
            &TokenType::GreaterEqual,
            &TokenType::Less,
            &TokenType::LessEqual,
        ]) {
            let operator = match self.previous().token_type {
                TokenType::Greater => BinaryOp::Greater,
                TokenType::GreaterEqual => BinaryOp::GreaterEqual,
                TokenType::Less => BinaryOp::Less,
                TokenType::LessEqual => BinaryOp::LessEqual,
                _ => unreachable!(),
            };
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        let mut expr = self.factor()?;

        while self.match_any(&[&TokenType::Plus, &TokenType::Minus]) {
            let operator = match self.previous().token_type {
                TokenType::Plus => BinaryOp::Add,
                TokenType::Minus => BinaryOp::Subtract,
                _ => unreachable!(),
            };
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        let mut expr = self.unary()?;

        while self.match_any(&[&TokenType::Star, &TokenType::Slash]) {
            let operator = match self.previous().token_type {
                TokenType::Star => BinaryOp::Multiply,
                TokenType::Slash => BinaryOp::Divide,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        if self.match_any(&[&TokenType::Bang, &TokenType::Minus]) {
            let operator = match self.previous().token_type {
                TokenType::Bang => UnaryOp::Not,
                TokenType::Minus => UnaryOp::Minus,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                expr: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr, Box<dyn Error + Send + Sync>> {
        if self.match_token(&TokenType::True) {
            Ok(Expr::Literal(Value::Bool(true)))
        } else if self.match_token(&TokenType::False) {
            Ok(Expr::Literal(Value::Bool(false)))
        } else if self.match_token(&TokenType::Null) {
            Ok(Expr::Literal(Value::Null))
        } else if let TokenType::Number(n) = self.peek().token_type {
            self.advance();
            Ok(Expr::Literal(Value::Number(n)))
        } else if let TokenType::String(s) = self.peek().token_type.clone() {
            self.advance();
            Ok(Expr::Literal(Value::String(s)))
        } else if let TokenType::Identifier(name) = self.peek().token_type.clone() {
            self.advance();
            Ok(Expr::Variable(name))
        } else if self.match_token(&TokenType::LeftParen) {
            let expr = self.expression()?;
            self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
            Ok(expr)
        } else {
            Err(Box::new(ParseError::new(
                self.peek(),
                "Expected expression".to_string(),
            )))
        }
    }

    fn match_token(&mut self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_any(&mut self, token_types: &[&TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::EOF)
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, Box<dyn Error + Send + Sync>> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(Box::new(ParseError::new(
                self.peek(),
                message.to_string(),
            )))
        }
    }

    fn consume_identifier(&mut self) -> Result<Token, Box<dyn Error + Send + Sync>> {
        if let TokenType::Identifier(_) = &self.peek().token_type {
            Ok(self.advance())
        } else {
            Err(Box::new(ParseError::new(
                self.peek(),
                "Expected identifier".to_string(),
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_let_declaration() {
        let tokens = vec![
            Token {
                token_type: TokenType::Let,
                lexeme: "let".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::Identifier("x".to_string()),
                lexeme: "x".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::Equals,
                lexeme: "=".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::Number(42.0),
                lexeme: "42".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::Semicolon,
                lexeme: ";".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: "".to_string(),
                line: 1,
            },
        ];

        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        
        assert_eq!(statements.len(), 1);
        match &statements[0] {
            Stmt::Let(name, expr) => {
                assert_eq!(name, "x");
                match expr {
                    Expr::Literal(Value::Number(n)) => assert_eq!(*n, 42.0),
                    _ => panic!("Expected number literal"),
                }
            }
            _ => panic!("Expected let statement"),
        }
    }

    #[test]
    fn test_parse_if_statement() {
        let tokens = vec![
            Token {
                token_type: TokenType::If,
                lexeme: "if".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::LeftParen,
                lexeme: "(".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::True,
                lexeme: "true".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::RightParen,
                lexeme: ")".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::LeftBrace,
                lexeme: "{".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::Return,
                lexeme: "return".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::Number(1.0),
                lexeme: "1".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::Semicolon,
                lexeme: ";".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::RightBrace,
                lexeme: "}".to_string(),
                line: 1,
            },
            Token {
                token_type: TokenType::EOF,
                lexeme: "".to_string(),
                line: 1,
            },
        ];

        let mut parser = Parser::new(tokens);
        let statements = parser.parse().unwrap();
        
        assert_eq!(statements.len(), 1);
        match &statements[0] {
            Stmt::If { condition, then_branch, else_branch } => {
                match condition {
                    Expr::Literal(Value::Bool(true)) => {},
                    _ => panic!("Expected true literal"),
                }
                match &**then_branch {
                    Stmt::Block(stmts) => {
                        assert_eq!(stmts.len(), 1);
                        match &stmts[0] {
                            Stmt::Return(expr) => {
                                match &**expr {
                                    Expr::Literal(Value::Number(1.0)) => {},
                                    _ => panic!("Expected number literal 1"),
                                }
                            },
                            _ => panic!("Expected return statement"),
                        }
                    },
                    _ => panic!("Expected block statement"),
                }
                assert!(else_branch.is_none());
            },
            _ => panic!("Expected if statement"),
        }
    }
}
