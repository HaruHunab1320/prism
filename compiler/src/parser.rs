use crate::ast::{BinaryOp, Expr, Stmt, UnaryOp, Value};
use crate::lexer::{Token, TokenType};
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

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, current: 0 }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&TokenType::Let) {
            self.var_declaration()
        } else if self.match_token(&TokenType::Function) {
            self.function_declaration(false)
        } else if self.match_token(&TokenType::Async) {
            if self.match_token(&TokenType::Function) {
                self.function_declaration(true)
            } else {
                Err(ParseError::new(&self.peek(), "Expected 'fn' after 'async'"))
            }
        } else {
            self.statement()
        }
    }

    fn var_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self
            .consume(
                &TokenType::Identifier(String::new()),
                "Expected variable name",
            )?
            .lexeme
            .clone();

        let initializer = if self.match_token(&TokenType::Equal) {
            Some(self.expression()?)
        } else {
            None
        };

        self.consume(
            &TokenType::Semicolon,
            "Expected ';' after variable declaration",
        )?;
        Ok(Stmt::Let(
            name,
            initializer.unwrap_or(Expr::Literal(Value::Null)),
        ))
    }

    fn function_declaration(&mut self, is_async: bool) -> Result<Stmt, ParseError> {
        let name = self
            .consume(
                &TokenType::Identifier(String::new()),
                "Expected function name",
            )?
            .lexeme
            .clone();
        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                if params.len() >= 255 {
                    return Err(ParseError::new(
                        &self.peek(),
                        "Cannot have more than 255 parameters",
                    ));
                }

                let param = self
                    .consume(
                        &TokenType::Identifier(String::new()),
                        "Expected parameter name",
                    )?
                    .lexeme
                    .clone();
                params.push(param);

                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;
        self.consume(&TokenType::LeftBrace, "Expected '{' before function body")?;
        let statements = self.block()?;

        Ok(Stmt::Function {
            name,
            params,
            body: Box::new(Stmt::Block(statements)),
            is_async,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&TokenType::If) {
            self.if_statement()
        } else if self.match_token(&TokenType::While) {
            self.while_statement()
        } else if self.match_token(&TokenType::For) {
            self.for_statement()
        } else if self.match_token(&TokenType::Return) {
            self.return_statement()
        } else if self.match_token(&TokenType::LeftBrace) {
            let statements = self.block()?;
            Ok(Stmt::Block(statements))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after if condition")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&TokenType::Else) {
            Some(Box::new(self.statement()?))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = self.expression()?;
        self.consume(&TokenType::RightParen, "Expected ')' after condition")?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn for_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'for'")?;

        let initializer = if self.match_token(&TokenType::Semicolon) {
            None
        } else if self.match_token(&TokenType::Let) {
            Some(self.var_declaration()?)
        } else {
            Some(self.expression_statement()?)
        };

        let condition = if !self.check(&TokenType::Semicolon) {
            self.expression()?
        } else {
            Expr::Literal(Value::Bool(true))
        };
        self.consume(&TokenType::Semicolon, "Expected ';' after loop condition")?;

        let increment = if !self.check(&TokenType::RightParen) {
            Some(self.expression()?)
        } else {
            None
        };
        self.consume(&TokenType::RightParen, "Expected ')' after for clauses")?;

        let mut body = self.statement()?;

        if let Some(increment) = increment {
            body = Stmt::Block(vec![body, Stmt::Expression(increment)]);
        }

        body = Stmt::While {
            condition,
            body: Box::new(body),
        };

        if let Some(initializer) = initializer {
            body = Stmt::Block(vec![initializer, body]);
        }

        Ok(body)
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, ParseError> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.or()?;

        if self.match_token(&TokenType::Equal) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Binary {
                    left: Box::new(Expr::Variable(name)),
                    operator: BinaryOp::Equal,
                    right: Box::new(value),
                });
            }

            return Err(ParseError::new(&equals, "Invalid assignment target."));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, ParseError> {
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

    fn and(&mut self) -> Result<Expr, ParseError> {
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

    fn equality(&mut self) -> Result<Expr, ParseError> {
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

    fn comparison(&mut self) -> Result<Expr, ParseError> {
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

    fn term(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.factor()?;

        while self.match_any(&[&TokenType::Minus, &TokenType::Plus]) {
            let operator = match self.previous().token_type {
                TokenType::Minus => BinaryOp::Subtract,
                TokenType::Plus => BinaryOp::Add,
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

    fn factor(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.unary()?;

        while self.match_any(&[&TokenType::Slash, &TokenType::Star]) {
            let operator = match self.previous().token_type {
                TokenType::Slash => BinaryOp::Divide,
                TokenType::Star => BinaryOp::Multiply,
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

    fn unary(&mut self) -> Result<Expr, ParseError> {
        if self.match_any(&[&TokenType::Bang, &TokenType::Minus]) {
            let operator = match self.previous().token_type {
                TokenType::Bang => UnaryOp::Not,
                TokenType::Minus => UnaryOp::Minus,
                _ => unreachable!(),
            };
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                expr: Box::new(right),
            });
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&TokenType::False) {
            Ok(Expr::Literal(Value::Bool(false)))
        } else if self.match_token(&TokenType::True) {
            Ok(Expr::Literal(Value::Bool(true)))
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
            Err(ParseError::new(&self.peek(), "Expected expression"))
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
        for &token_type in token_types {
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

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token_type: &TokenType, message: &str) -> Result<Token, ParseError> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(ParseError::new(self.peek(), message))
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, ParseError> {
        let value = if self.check(&TokenType::Semicolon) {
            Expr::Literal(Value::Null)
        } else {
            self.expression()?
        };

        self.consume(&TokenType::Semicolon, "Expected ';' after return value")?;
        Ok(Stmt::Return(Box::new(value)))
    }
}
