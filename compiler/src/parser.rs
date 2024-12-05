use crate::ast::{Expr, Stmt};
use crate::lexer::Token;
use crate::value::Value;
use std::mem;

pub struct Parser {
    source: String,
    tokens: Vec<Token>,
    starts: Vec<usize>,
    ends: Vec<usize>,
    current: usize,
}

impl Parser {
    pub fn new(source: String, tokens: Vec<Token>, starts: Vec<usize>, ends: Vec<usize>) -> Self {
        Self {
            source,
            tokens,
            starts,
            ends,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, String> {
        if self.match_token(&[Token::Let]) {
            self.let_statement()
        } else if self.match_token(&[Token::LeftBrace]) {
            self.block_statement()
        } else if self.match_token(&[Token::Context]) {
            self.context_statement()
        } else if self.match_token(&[Token::Verify]) {
            self.verify_statement()
        } else if self.match_token(&[Token::Function]) {
            self.function_statement()
        } else {
            Ok(Stmt::Expression(self.expression()?))
        }
    }

    fn let_statement(&mut self) -> Result<Stmt, String> {
        let name = self.consume_identifier("Expected variable name.")?;
        self.consume(&Token::Equal, "Expected '=' after variable name.")?;
        let initializer = self.expression()?;
        self.consume(&Token::Semicolon, "Expected ';' after variable declaration.")?;
        Ok(Stmt::Let(name, initializer))
    }

    fn block_statement(&mut self) -> Result<Stmt, String> {
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        self.consume(&Token::RightBrace, "Expected '}' after block.")?;
        Ok(Stmt::Block(statements))
    }

    fn context_statement(&mut self) -> Result<Stmt, String> {
        let name = self.consume_identifier("Expected context name.")?;
        self.consume(&Token::LeftBrace, "Expected '{' after context name.")?;
        let body = Box::new(self.block_statement()?);
        Ok(Stmt::Context(name, body))
    }

    fn verify_statement(&mut self) -> Result<Stmt, String> {
        self.consume(&Token::Against, "Expected 'against' after 'verify'.")?;
        self.consume(&Token::Sources, "Expected 'sources' after 'against'.")?;
        self.consume(&Token::LeftBracket, "Expected '[' after 'sources'.")?;

        let mut sources = Vec::new();
        if !self.check(&Token::RightBracket) {
            loop {
                sources.push(self.consume_string("Expected source name.")?);
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
        self.consume(&Token::RightBracket, "Expected ']' after sources.")?;
        self.consume(&Token::LeftBrace, "Expected '{' after sources.")?;
        let body = Box::new(self.block_statement()?);
        Ok(Stmt::Verify(sources, body))
    }

    fn function_statement(&mut self) -> Result<Stmt, String> {
        let is_async = self.match_token(&[Token::Async]);
        let name = self.consume_identifier("Expected function name.")?;
        self.consume(&Token::LeftParen, "Expected '(' after function name.")?;

        let mut params = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                params.push(self.consume_identifier("Expected parameter name.")?);
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
        self.consume(&Token::RightParen, "Expected ')' after parameters.")?;
        self.consume(&Token::LeftBrace, "Expected '{' before function body.")?;
        let body = Box::new(self.block_statement()?);

        Ok(Stmt::Function {
            name,
            params,
            body,
            is_async,
        })
    }

    fn expression(&mut self) -> Result<Expr, String> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, String> {
        let expr = self.or()?;

        if self.match_token(&[Token::Equal]) {
            let equals = self.previous();
            let value = Box::new(self.assignment()?);

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign(name, value));
            }

            return Err(format!("Invalid assignment target at position {}.", self.starts[equals]));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, String> {
        let mut expr = self.and()?;

        while self.match_token(&[Token::Or]) {
            let operator = self.previous_lexeme();
            let right = Box::new(self.and()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, String> {
        let mut expr = self.equality()?;

        while self.match_token(&[Token::And]) {
            let operator = self.previous_lexeme();
            let right = Box::new(self.equality()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, String> {
        let mut expr = self.comparison()?;

        while self.match_token(&[Token::EqualEqual, Token::BangEqual]) {
            let operator = self.previous_lexeme();
            let right = Box::new(self.comparison()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, String> {
        let mut expr = self.term()?;

        while self.match_token(&[Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual]) {
            let operator = self.previous_lexeme();
            let right = Box::new(self.term()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, String> {
        let mut expr = self.factor()?;

        while self.match_token(&[Token::Plus, Token::Minus]) {
            let operator = self.previous_lexeme();
            let right = Box::new(self.factor()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, String> {
        let mut expr = self.unary()?;

        while self.match_token(&[Token::Star, Token::Slash]) {
            let operator = self.previous_lexeme();
            let right = Box::new(self.unary()?);
            expr = Expr::Binary(Box::new(expr), operator, right);
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.match_token(&[Token::Bang, Token::Minus]) {
            let operator = self.previous_lexeme();
            let right = Box::new(self.unary()?);
            return Ok(Expr::Unary(operator, right));
        }

        self.call()
    }

    fn call(&mut self) -> Result<Expr, String> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[Token::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[Token::Tilde]) {
                if self.match_token(&[Token::Greater]) {
                    let confidence = self.consume_number("Expected confidence value after '~>'.")?;
                    expr = Expr::Confidence(Box::new(expr), confidence);
                } else if self.match_token(&[Token::Equal]) {
                    let target = Box::new(self.expression()?);
                    expr = Expr::SemanticMatch(Box::new(expr), target);
                }
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, String> {
        let mut arguments = Vec::new();
        if !self.check(&Token::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }

        self.consume(&Token::RightParen, "Expected ')' after arguments.")?;
        Ok(Expr::Call(Box::new(callee), arguments))
    }

    fn primary(&mut self) -> Result<Expr, String> {
        if self.match_token(&[Token::False]) {
            Ok(Expr::Literal(Value::Boolean(false)))
        } else if self.match_token(&[Token::True]) {
            Ok(Expr::Literal(Value::Boolean(true)))
        } else if self.match_token(&[Token::None]) {
            Ok(Expr::Literal(Value::None))
        } else if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            Ok(Expr::Literal(Value::Float(n)))
        } else if let Some(Token::String(s)) = self.current_token() {
            self.advance();
            Ok(Expr::Literal(Value::String(s)))
        } else if let Some(Token::Identifier(name)) = self.current_token() {
            self.advance();
            Ok(Expr::Variable(name))
        } else if self.match_token(&[Token::LeftParen]) {
            let expr = self.expression()?;
            self.consume(&Token::RightParen, "Expected ')' after expression.")?;
            Ok(expr)
        } else if self.match_token(&[Token::Await]) {
            let expr = Box::new(self.expression()?);
            Ok(Expr::Await(expr))
        } else {
            Err(format!("Expected expression at position {}.", self.starts[self.current]))
        }
    }

    fn match_token(&mut self, tokens: &[Token]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token: &Token) -> bool {
        if self.is_at_end() {
            false
        } else {
            match (self.current_token(), token) {
                (Some(Token::Number(_)), Token::Number(_)) => true,
                (Some(Token::String(_)), Token::String(_)) => true,
                (Some(Token::Identifier(_)), Token::Identifier(_)) => true,
                (Some(t1), t2) => mem::discriminant(&t1) == mem::discriminant(t2),
                _ => false,
            }
        }
    }

    fn advance(&mut self) -> usize {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn previous(&self) -> usize {
        self.current - 1
    }

    fn current_token(&self) -> Option<Token> {
        if self.is_at_end() {
            None
        } else {
            Some(self.tokens[self.current].clone())
        }
    }

    fn previous_lexeme(&self) -> String {
        let start = self.starts[self.current - 1];
        let end = self.ends[self.current - 1];
        self.source[start..end].to_string()
    }

    fn consume(&mut self, token: &Token, message: &str) -> Result<(), String> {
        if self.check(token) {
            self.advance();
            Ok(())
        } else {
            Err(format!("{} at position {}.", message, self.starts[self.current]))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, String> {
        if let Some(Token::Identifier(name)) = self.current_token() {
            self.advance();
            Ok(name)
        } else {
            Err(format!("{} at position {}.", message, self.starts[self.current]))
        }
    }

    fn consume_string(&mut self, message: &str) -> Result<String, String> {
        if let Some(Token::String(s)) = self.current_token() {
            self.advance();
            Ok(s)
        } else {
            Err(format!("{} at position {}.", message, self.starts[self.current]))
        }
    }

    fn consume_number(&mut self, message: &str) -> Result<f64, String> {
        if let Some(Token::Number(n)) = self.current_token() {
            self.advance();
            Ok(n)
        } else {
            Err(format!("{} at position {}.", message, self.starts[self.current]))
        }
    }
} 