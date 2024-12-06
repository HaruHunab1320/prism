use crate::ast::{Expr, Stmt};
use crate::error::Error;
use crate::lexer::{Token, TokenType};
use crate::value::Value;
use std::cell::RefCell;

pub struct Parser {
    tokens: Vec<Token>,
    current: RefCell<usize>,
}

impl Parser {
    pub fn new(tokens: Vec<Token>) -> Self {
        Self {
            tokens,
            current: RefCell::new(0),
        }
    }

    // Program -> Declaration*
    pub fn parse(&mut self) -> Result<Vec<Stmt>, Error> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            if let Ok(stmt) = self.declaration() {
                statements.push(stmt);
            } else {
                // Skip to the next statement on error
                self.synchronize();
            }
        }
        Ok(statements)
    }

    // Declaration -> FunctionDecl | AsyncFunctionDecl | LetDecl | Statement
    fn declaration(&mut self) -> Result<Stmt, Error> {
        if self.match_token(&TokenType::Fn) {
            self.function_declaration(false)
        } else if self.match_token(&TokenType::Async) {
            if self.match_token(&TokenType::Fn) {
                self.function_declaration(true)
            } else {
                Err(Error::new("Expected 'fn' after 'async'"))
            }
        } else if self.match_token(&TokenType::Let) {
            self.let_declaration()
        } else {
            self.statement()
        }
    }

    // Statement -> ExprStmt | IfStmt | WhileStmt | ForStmt | ReturnStmt | Block
    fn statement(&mut self) -> Result<Stmt, Error> {
        if self.match_token(&TokenType::If) {
            self.if_statement()
        } else if self.match_token(&TokenType::While) {
            self.while_statement()
        } else if self.match_token(&TokenType::For) {
            self.for_statement()
        } else if self.match_token(&TokenType::Return) {
            self.return_statement()
        } else if self.match_token(&TokenType::LeftBrace) {
            self.block()
        } else {
            self.expression_statement()
        }
    }

    // Block -> "{" Declaration* "}"
    fn block(&mut self) -> Result<Stmt, Error> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenType::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        
        self.consume(&TokenType::RightBrace, "Expected '}' after block")?;
        Ok(Stmt::Block(statements))
    }

    // Error recovery
    fn synchronize(&mut self) {
        self.advance();

        while !self.is_at_end() {
            if self.previous().token_type == TokenType::Semicolon {
                return;
            }

            match self.peek().token_type {
                TokenType::Fn | 
                TokenType::Let | 
                TokenType::For | 
                TokenType::If | 
                TokenType::While | 
                TokenType::Return => return,
                _ => {
                    self.advance();
                }
            }
        }
    }

    // Utility methods
    fn match_token(&self, token_type: &TokenType) -> bool {
        if self.check(token_type) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn check(&self, token_type: &TokenType) -> bool {
        if self.is_at_end() {
            false
        } else {
            &self.peek().token_type == token_type
        }
    }

    fn advance(&self) -> Token {
        if !self.is_at_end() {
            let mut current = self.current.borrow_mut();
            *current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().token_type, TokenType::EOF)
    }

    fn peek(&self) -> Token {
        self.tokens[*self.current.borrow()].clone()
    }

    fn previous(&self) -> Token {
        let current = *self.current.borrow();
        self.tokens[current - 1].clone()
    }

    fn consume(&self, token_type: &TokenType, message: &str) -> Result<Token, Error> {
        if self.check(token_type) {
            Ok(self.advance())
        } else {
            Err(Error::new(message))
        }
    }

    // Expression -> Assignment
    fn expression(&mut self) -> Result<Expr, Error> {
        self.assignment()
    }

    // Assignment -> IDENTIFIER "=" Assignment | LogicalOr
    fn assignment(&mut self) -> Result<Expr, Error> {
        let expr = self.logical_or()?;

        if self.match_token(&TokenType::Equals) {
            let value = self.assignment()?;

            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value: Box::new(value),
                });
            }

            return Err(Error::new("Invalid assignment target"));
        }

        Ok(expr)
    }

    // LogicalOr -> LogicalAnd ("or" LogicalAnd)*
    fn logical_or(&mut self) -> Result<Expr, Error> {
        let mut expr = self.logical_and()?;

        while self.match_token(&TokenType::Or) {
            let operator = self.previous();
            let right = self.logical_and()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // LogicalAnd -> Equality ("and" Equality)*
    fn logical_and(&mut self) -> Result<Expr, Error> {
        let mut expr = self.equality()?;

        while self.match_token(&TokenType::And) {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Logical {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // Equality -> Comparison (("!=" | "==") Comparison)*
    fn equality(&mut self) -> Result<Expr, Error> {
        let mut expr = self.comparison()?;

        while self.match_any(&[TokenType::BangEqual, TokenType::EqualEqual]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // Comparison -> Term ((">" | ">=" | "<" | "<=") Term)*
    fn comparison(&mut self) -> Result<Expr, Error> {
        let mut expr = self.term()?;

        while self.match_any(&[
            TokenType::Greater,
            TokenType::GreaterEqual,
            TokenType::Less,
            TokenType::LessEqual,
        ]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // Term -> Factor (("+" | "-") Factor)*
    fn term(&mut self) -> Result<Expr, Error> {
        let mut expr = self.factor()?;

        while self.match_any(&[TokenType::Plus, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // Factor -> Unary (("*" | "/") Unary)*
    fn factor(&mut self) -> Result<Expr, Error> {
        let mut expr = self.unary()?;

        while self.match_any(&[TokenType::Star, TokenType::Slash]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator,
                right: Box::new(right),
            };
        }

        Ok(expr)
    }

    // Unary -> ("!" | "-") Unary | Call
    fn unary(&mut self) -> Result<Expr, Error> {
        if self.match_any(&[TokenType::Bang, TokenType::Minus]) {
            let operator = self.previous();
            let right = self.unary()?;
            return Ok(Expr::Unary {
                operator,
                right: Box::new(right),
            });
        }

        self.call()
    }

    // Call -> Primary ("(" Arguments? ")" | "." IDENTIFIER)*
    fn call(&mut self) -> Result<Expr, Error> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token(&TokenType::LeftParen) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&TokenType::Dot) {
                let name = self.identifier()?;
                expr = Expr::Get {
                    object: Box::new(expr),
                    name,
                };
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
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }

        self.consume(&TokenType::RightParen, "Expected ')' after arguments")?;
        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    // Primary -> NUMBER | STRING | "true" | "false" | "null" | IDENTIFIER | "(" Expression ")"
    fn primary(&mut self) -> Result<Expr, Error> {
        let token = self.peek();
        match token.token_type {
            TokenType::False => {
                self.advance();
                Ok(Expr::Literal(Value::Bool(false)))
            }
            TokenType::True => {
                self.advance();
                Ok(Expr::Literal(Value::Bool(true)))
            }
            TokenType::Null => {
                self.advance();
                Ok(Expr::Literal(Value::Null))
            }
            TokenType::Number(n) => {
                self.advance();
                Ok(Expr::Literal(Value::Number(n)))
            }
            TokenType::String(ref s) => {
                self.advance();
                Ok(Expr::Literal(Value::String(s.clone())))
            }
            TokenType::Identifier(ref name) => {
                self.advance();
                Ok(Expr::Variable(name.clone()))
            }
            TokenType::LeftParen => {
                self.advance();
                let expr = self.expression()?;
                self.consume(&TokenType::RightParen, "Expected ')' after expression")?;
                Ok(Expr::Grouping(Box::new(expr)))
            }
            _ => Err(Error::new("Expected expression")),
        }
    }

    fn match_any(&self, token_types: &[TokenType]) -> bool {
        for token_type in token_types {
            if self.check(token_type) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn identifier(&mut self) -> Result<String, Error> {
        if let TokenType::Identifier(name) = &self.peek().token_type {
            let name = name.clone();
            self.advance();
            Ok(name)
        } else {
            Err(Error::new("Expected identifier"))
        }
    }

    fn function_declaration(&mut self, is_async: bool) -> Result<Stmt, Error> {
        let name = self.identifier()?;
        self.consume(&TokenType::LeftParen, "Expected '(' after function name")?;

        let mut params = Vec::new();
        if !self.check(&TokenType::RightParen) {
            loop {
                params.push(self.identifier()?);
                if !self.match_token(&TokenType::Comma) {
                    break;
                }
            }
        }
        self.consume(&TokenType::RightParen, "Expected ')' after parameters")?;
        self.consume(&TokenType::LeftBrace, "Expected '{' before function body")?;

        let body = self.block()?;
        Ok(Stmt::Function {
            name,
            params,
            body: Box::new(body),
            is_async,
        })
    }

    fn let_declaration(&mut self) -> Result<Stmt, Error> {
        let name = self.identifier()?;
        
        let initializer = if self.match_token(&TokenType::Equals) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        
        self.consume(&TokenType::Semicolon, "Expected ';' after variable declaration")?;
        Ok(Stmt::Let(name, initializer))
    }

    fn if_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'if'")?;
        let condition = Box::new(self.expression()?);
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

    fn while_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'while'")?;
        let condition = Box::new(self.expression()?);
        self.consume(&TokenType::RightParen, "Expected ')' after while condition")?;

        let body = Box::new(self.statement()?);
        Ok(Stmt::While {
            condition,
            body,
        })
    }

    fn for_statement(&mut self) -> Result<Stmt, Error> {
        self.consume(&TokenType::LeftParen, "Expected '(' after 'for'")?;
        
        let initializer = if self.match_token(&TokenType::Semicolon) {
            None
        } else if self.match_token(&TokenType::Let) {
            Some(self.let_declaration()?)
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
        
        if let Some(inc) = increment {
            body = Stmt::Block(vec![
                body,
                Stmt::Expression(Box::new(inc)),
            ]);
        }
        
        body = Stmt::While {
            condition: Box::new(condition),
            body: Box::new(body),
        };
        
        if let Some(init) = initializer {
            body = Stmt::Block(vec![init, body]);
        }
        
        Ok(body)
    }

    fn return_statement(&mut self) -> Result<Stmt, Error> {
        let value = if !self.check(&TokenType::Semicolon) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        
        self.consume(&TokenType::Semicolon, "Expected ';' after return value")?;
        Ok(Stmt::Return(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt, Error> {
        let expr = self.expression()?;
        self.consume(&TokenType::Semicolon, "Expected ';' after expression")?;
        Ok(Stmt::Expression(Box::new(expr)))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_function_declaration() {
        let source = r#"
            fn add(x, y) {
                x + y
            }
        "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        println!("Function tokens: {:?}", tokens);
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        if let Err(e) = &result {
            println!("Function declaration error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_async_function() {
        let source = r#"
            async fn fetch(url) {
                http.get(url)
            }
        "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        println!("Async function tokens: {:?}", tokens);
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        if let Err(e) = &result {
            println!("Async function error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_let_declaration() {
        let source = r#"
            let x = 42;
        "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        println!("Let declaration tokens: {:?}", tokens);
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        if let Err(e) = &result {
            println!("Let declaration error: {}", e);
        }
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_if_statement() {
        let source = r#"
            if x > 0 {
                let y = x + 1;
                y
            }
        "#;
        let mut lexer = Lexer::new(source);
        let tokens = lexer.lex().unwrap();
        println!("If statement tokens: {:?}", tokens);
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        if let Err(e) = &result {
            println!("If statement error: {}", e);
        }
        assert!(result.is_ok());
    }
}
