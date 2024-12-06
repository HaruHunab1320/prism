use crate::ast::{Expr, Stmt};
use crate::error::Error;
use crate::lexer::{Token, TokenType};
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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, Error> {
        if self.match_token(&[TokenType::Let]) {
            self.let_declaration()
        } else if self.match_token(&[TokenType::Fn]) {
            self.function_declaration()
        } else {
            self.statement()
        }
    }

    fn function_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(TokenType::Identifier, "Expected function name.")?
            .lexeme
            .clone();

        let mut confidence = None;
        if self.match_token(&[TokenType::Confidence]) {
            confidence = Some(
                self.consume(TokenType::Number, "Expected confidence value.")?
                    .lexeme
                    .parse()
                    .map_err(|_| Error::new("Invalid confidence value"))?,
            );
        }

        self.consume(TokenType::LeftParen, "Expected '(' after function name.")?;
        let mut parameters = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                parameters.push(
                    self.consume(TokenType::Identifier, "Expected parameter name.")?
                        .lexeme
                        .clone(),
                );

                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(TokenType::RightParen, "Expected ')' after parameters.")?;
        self.consume(TokenType::LeftBrace, "Expected '{' before function body.")?;
        let body = Box::new(self.block_statement()?);

        Ok(Stmt::Function {
            name,
            params: parameters,
            body,
            is_async: false,
            confidence,
        })
    }

    fn let_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.consume(TokenType::Identifier, "Expected variable name.")?
            .lexeme
            .clone();

        let initializer = if self.match_token(&[TokenType::Equal]) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };

        self.consume(TokenType::Semicolon, "Expected ';' after variable declaration.")?;
        Ok(Stmt::Let(name, initializer))
    }

    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.match_token(&[TokenType::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenType::UncertainIf]) {
            self.uncertain_if_statement()
        } else if self.match_token(&[TokenType::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenType::In]) {
            self.context_statement()
        } else if self.match_token(&[TokenType::LeftBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'if'.")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expected ')' after if condition.")?;

        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[TokenType::Else]) {
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

    fn uncertain_if_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'uncertain if'.")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expected ')' after if condition.")?;

        self.consume(TokenType::LeftBrace, "Expected '{' after uncertain if condition.")?;
        let then_branch = Box::new(self.block_statement()?);

        let mut medium_branch = None;
        let mut low_branch = None;

        if self.match_token(&[TokenType::Medium]) {
            self.consume(TokenType::LeftBrace, "Expected '{' after medium.")?;
            medium_branch = Some(Box::new(self.block_statement()?));
        }

        if self.match_token(&[TokenType::Low]) {
            self.consume(TokenType::LeftBrace, "Expected '{' after low.")?;
            low_branch = Some(Box::new(self.block_statement()?));
        }

        Ok(Stmt::UncertainIf {
            condition,
            then_branch,
            medium_branch,
            low_branch,
        })
    }

    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::LeftParen, "Expected '(' after 'while'.")?;
        let condition = Box::new(self.expression()?);
        self.consume(TokenType::RightParen, "Expected ')' after condition.")?;
        let body = Box::new(self.statement()?);

        Ok(Stmt::While { condition, body })
    }

    fn context_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(TokenType::Context, "Expected 'context' after 'in'.")?;
        let name = self.consume(TokenType::Identifier, "Expected context name.")?
            .lexeme
            .clone();

        self.consume(TokenType::LeftBrace, "Expected '{' after context name.")?;
        let body = Box::new(self.block_statement()?);

        Ok(Stmt::Context { name, body })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::new();

        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }

        self.consume(TokenType::RightBrace, "Expected '}' after block.")?;
        Ok(statements)
    }

    fn block_statement(&mut self) -> Result<Stmt, Error> {
        Ok(Stmt::Block(self.block()?))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(TokenType::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression(Box::new(expr)))
    }

    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.or()?;

        if self.match_token(&[TokenType::Equal]) {
            let equals = self.previous().clone();
            let value = Box::new(self.assignment()?);

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign { name, value });
            }

            return Err(Error::new(&format!(
                "Invalid assignment target at line {}.",
                equals.line
            )));
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.and()?;

        while self.match_token(&[TokenType::Or]) {
            let operator = self.previous().clone();
            let right = Box::new(self.and()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while self.match_token(&[TokenType::And]) {
            let operator = self.previous().clone();
            let right = Box::new(self.equality()?);
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_token(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous().clone();
            let right = Box::new(self.comparison()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.match_token(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous().clone();
            let right = Box::new(self.term()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.match_token(&[TokenType::Minus, TokenType::Plus]) {
            let operator = self.previous().clone();
            let right = Box::new(self.factor()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.match_token(&[TokenType::Slash, TokenType::Star]) {
            let operator = self.previous().clone();
            let right = Box::new(self.unary()?);
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right,
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_token(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous().clone();
            let right = Box::new(self.unary()?);
            Ok(Expr::Unary { operator, right })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, Error> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&[TokenType::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, Error> {
        let mut arguments = Vec::new();

        if !self.check(&TokenType::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenType::Comma]) {
                    break;
                }
            }
        }

        self.consume(
            TokenType::RightParen,
            "Expected ')' after function arguments.",
        )?;

        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, Error> {
        if self.match_token(&[TokenType::False]) {
            Ok(Expr::Literal(Value::new(ValueKind::Bool(false))))
        } else if self.match_token(&[TokenType::True]) {
            Ok(Expr::Literal(Value::new(ValueKind::Bool(true))))
        } else if self.match_token(&[TokenType::Nil]) {
            Ok(Expr::Literal(Value::new(ValueKind::Null)))
        } else if self.match_token(&[TokenType::Number]) {
            let value = self.previous().lexeme.parse()
                .map_err(|_| Error::new("Invalid number literal"))?;
            Ok(Expr::Literal(Value::new(ValueKind::Number(value))))
        } else if self.match_token(&[TokenType::String]) {
            let value = self.previous().lexeme.clone();
            Ok(Expr::Literal(Value::new(ValueKind::String(value))))
        } else if self.match_token(&[TokenType::Identifier]) {
            Ok(Expr::Variable(self.previous().lexeme.clone()))
        } else if self.match_token(&[TokenType::LeftParen]) {
            let expr = self.expression()?;
            self.consume(TokenType::RightParen, "Expected ')' after expression.")?;
            Ok(Expr::Grouping(Box::new(expr)))
        } else {
            Err(Error::new(&format!(
                "Expected expression at line {}.",
                self.peek().line
            )))
        }
    }

    fn match_token(&mut self, types: &[TokenType]) -> bool {
        for token_type in types {
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
        self.peek().token_type == TokenType::EOF
    }

    fn peek(&self) -> Token {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token_type: TokenType, message: &str) -> Result<Token, Error> {
        if self.check(&token_type) {
            Ok(self.advance())
        } else {
            Err(Error::new(&format!(
                "{} at line {}",
                message,
                self.peek().line
            )))
        }
    }

    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Class
                | TokenType::Fun
                | TokenType::Let
                | TokenType::For
                | TokenType::If
                | TokenType::While
                | TokenType::Return => return,
                _ => (),
            }

            self.advance();
        }
    }
}
