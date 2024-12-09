use crate::ast::{Expr, Stmt};
use crate::error::{PrismError, Result};
use crate::token::{Token, TokenKind};
use crate::lexer::Lexer;
use crate::value::{Value, ValueKind};

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenKind::Let]) {
            self.let_declaration()
        } else if self.match_token(&[TokenKind::Fun]) {
            self.function_declaration()
        } else {
            self.statement()
        }
    }

    fn let_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier("Expected variable name.")?;
        
        let initializer = if self.match_token(&[TokenKind::Equal]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.consume(TokenKind::Semicolon, "Expected ';' after variable declaration.")?;
        Ok(Stmt::Let(name, initializer))
    }

    fn function_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier("Expected function name.")?;
        self.consume(TokenKind::LeftParen, "Expected '(' after function name.")?;
        
        let mut params = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                params.push(self.consume_identifier("Expected parameter name.")?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(TokenKind::RightParen, "Expected ')' after parameters.")?;
        
        let is_async = self.match_token(&[TokenKind::Async]);
        let confidence = if self.match_token(&[TokenKind::Confidence]) {
            Some(self.consume_number("Expected confidence value.")?)
        } else {
            None
        };
        
        self.consume(TokenKind::LeftBrace, "Expected '{' before function body.")?;
        let body = Box::new(self.block()?);
        
        Ok(Stmt::Function { name, params, body, is_async, confidence })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenKind::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenKind::LeftBrace]) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(TokenKind::LeftParen, "Expected '(' after 'if'.")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenKind::RightParen, "Expected ')' after if condition.")?;

        let then_branch = Box::new(self.block()?);
        let else_branch = if self.match_token(&[TokenKind::Else]) {
            Some(Box::new(if self.match_token(&[TokenKind::If]) {
                self.if_statement()?
            } else {
                self.block()?
            }))
        } else {
            None
        };

        Ok(Stmt::If {
            condition,
            then_branch,
            else_branch,
        })
    }

    fn block(&mut self) -> Result<Stmt> {
        self.consume(TokenKind::LeftBrace, "Expected '{' before block.")?;
        let mut statements = Vec::new();

        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenKind::RightBrace, "Expected '}' after block.")?;
        Ok(Stmt::Block(statements))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(TokenKind::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression(Box::new(expr)))
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.equality()?;

        if self.match_token(&[TokenKind::Equal]) {
            let equals = self.previous().clone();
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            return Err(PrismError::ParseError(format!(
                "Invalid assignment target at line {}.",
                equals.line
            )));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
            let operator = self.previous().clone();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenKind::Plus, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenKind::Star, TokenKind::Slash]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = self.unary()?;
            Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            })
        } else {
            self.primary()
        }
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenKind::False]) {
            Ok(Expr::Literal(Value::new(ValueKind::Boolean(false))))
        } else if self.match_token(&[TokenKind::True]) {
            Ok(Expr::Literal(Value::new(ValueKind::Boolean(true))))
        } else if self.match_token(&[TokenKind::Nil]) {
            Ok(Expr::Literal(Value::new(ValueKind::Nil)))
        } else if self.check_number() {
            self.advance();
            if let TokenKind::Number(n) = self.previous().kind {
                Ok(Expr::Literal(Value::new(ValueKind::Number(n))))
            } else {
                unreachable!()
            }
        } else if self.match_token(&[TokenKind::String(String::new())]) {
            if let TokenKind::String(ref s) = self.previous().kind {
                Ok(Expr::Literal(Value::new(ValueKind::String(s.clone()))))
            } else {
                unreachable!()
            }
        } else if self.match_token(&[TokenKind::Identifier(String::new())]) {
            if let TokenKind::Identifier(ref name) = self.previous().kind {
                Ok(Expr::Variable(name.clone()))
            } else {
                unreachable!()
            }
        } else if self.match_token(&[TokenKind::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenKind::RightParen, "Expected ')' after expression.")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(PrismError::ParseError(format!(
                "Expected expression at line {}",
                self.peek().line
            )))
        }
    }

    fn match_token(&mut self, kinds: &[TokenKind]) -> bool {
        for kind in kinds {
            if self.check(kind) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, kind: &TokenKind) -> bool {
        if self.is_at_end() {
            false
        } else {
            match (kind, &self.peek().kind) {
                (TokenKind::Number(_), TokenKind::Number(_)) => true,
                (TokenKind::String(_), TokenKind::String(_)) => true,
                (TokenKind::Identifier(_), TokenKind::Identifier(_)) => true,
                (k1, k2) => std::mem::discriminant(k1) == std::mem::discriminant(k2),
            }
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.peek().kind == TokenKind::EOF
    }

    fn peek(&self) -> &Token {
        &self.tokens[self.current]
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }

    fn consume(&mut self, kind: TokenKind, message: &str) -> Result<&Token> {
        if self.check(&kind) {
            Ok(self.advance())
        } else {
            Err(PrismError::ParseError(message.to_string()))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String> {
        if let TokenKind::Identifier(ref name) = self.peek().kind {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(PrismError::ParseError(message.to_string()))
        }
    }

    fn consume_number(&mut self, message: &str) -> Result<f64> {
        if let TokenKind::Number(n) = self.peek().kind {
            let n = n;
            self.advance();
            Ok(n)
        } else {
            Err(PrismError::ParseError(message.to_string()))
        }
    }

    fn check_number(&self) -> bool {
        match &self.peek().kind {
            TokenKind::Number(_) => true,
            _ => false,
        }
    }
}

pub fn parse(source: &str) -> Result<Vec<Stmt>> {
    let mut lexer = Lexer::new(source.to_string());
    let tokens = lexer.scan_tokens()?;
    let mut parser = Parser::new(tokens);
    parser.parse()
}
