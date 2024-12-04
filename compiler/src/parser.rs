use crate::ast::{Expr, Stmt};
use crate::error::RuntimeError;
use std::sync::Arc;

pub struct Parser {
    tokens: Vec<String>,
    current: usize,
}

impl Parser {
    pub fn new(tokens: Vec<String>) -> Self {
        Parser {
            tokens,
            current: 0,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Arc<Stmt>>, RuntimeError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(Arc::new(self.declaration()?));
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, RuntimeError> {
        if self.match_token("let") {
            self.let_declaration()
        } else if self.match_token("fn") {
            self.function_declaration()
        } else {
            self.statement()
        }
    }

    fn let_declaration(&mut self) -> Result<Stmt, RuntimeError> {
        let name = self.consume_identifier("Expected variable name")?;
        self.consume("=", "Expected '=' after variable name")?;
        let initializer = Arc::new(self.expression()?);
        self.consume(";", "Expected ';' after variable declaration")?;
        Ok(Stmt::Let { name, initializer })
    }

    fn function_declaration(&mut self) -> Result<Stmt, RuntimeError> {
        let name = self.consume_identifier("Expected function name")?;
        self.consume("(", "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(")") {
            loop {
                params.push(self.consume_identifier("Expected parameter name")?);
                if !self.match_token(",") {
                    break;
                }
            }
        }
        self.consume(")", "Expected ')' after parameters")?;
        self.consume("{", "Expected '{' before function body")?;
        let statements = self.block()?;
        let body = Arc::new(Stmt::Block(statements));
        Ok(Stmt::Function { name, params, body })
    }

    fn statement(&mut self) -> Result<Stmt, RuntimeError> {
        if self.match_token("return") {
            self.return_statement()
        } else if self.match_token("throw") {
            self.throw_statement()
        } else if self.match_token("{") {
            let statements = self.block()?;
            Ok(Stmt::Block(statements))
        } else {
            self.expression_statement()
        }
    }

    fn return_statement(&mut self) -> Result<Stmt, RuntimeError> {
        let value = Arc::new(self.expression()?);
        self.consume(";", "Expected ';' after return value")?;
        Ok(Stmt::Return(value))
    }

    fn throw_statement(&mut self) -> Result<Stmt, RuntimeError> {
        let value = Arc::new(self.expression()?);
        self.consume(";", "Expected ';' after throw value")?;
        Ok(Stmt::Throw(value))
    }

    fn block(&mut self) -> Result<Vec<Arc<Stmt>>, RuntimeError> {
        let mut statements = Vec::new();
        while !self.check("}") && !self.is_at_end() {
            statements.push(Arc::new(self.declaration()?));
        }
        self.consume("}", "Expected '}' after block")?;
        Ok(statements)
    }

    fn expression_statement(&mut self) -> Result<Stmt, RuntimeError> {
        let expr = Arc::new(self.expression()?);
        self.consume(";", "Expected ';' after expression")?;
        Ok(Stmt::Expression(expr))
    }

    fn expression(&mut self) -> Result<Expr, RuntimeError> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr, RuntimeError> {
        let expr = self.or()?;

        if self.match_token("=") {
            let _equals = self.previous();
            let value = self.assignment()?;

            match expr {
                Expr::Identifier(name) => {
                    return Ok(Expr::Binary {
                        left: Arc::new(Expr::Identifier(name)),
                        operator: "=".to_string(),
                        right: Arc::new(value),
                    });
                }
                _ => return Err(RuntimeError::TypeError("Invalid assignment target.".to_string())),
            }
        }

        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.and()?;

        while self.match_token("||") {
            let operator = self.previous();
            let right = self.and()?;
            expr = Expr::Binary {
                left: Arc::new(expr),
                operator,
                right: Arc::new(right),
            };
        }

        Ok(expr)
    }

    fn and(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.equality()?;

        while self.match_token("&&") {
            let operator = self.previous();
            let right = self.equality()?;
            expr = Expr::Binary {
                left: Arc::new(expr),
                operator,
                right: Arc::new(right),
            };
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.comparison()?;

        while self.match_any(&["==", "!="]) {
            let operator = self.previous();
            let right = self.comparison()?;
            expr = Expr::Binary {
                left: Arc::new(expr),
                operator,
                right: Arc::new(right),
            };
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.term()?;

        while self.match_any(&["<", "<=", ">", ">="]) {
            let operator = self.previous();
            let right = self.term()?;
            expr = Expr::Binary {
                left: Arc::new(expr),
                operator,
                right: Arc::new(right),
            };
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.factor()?;

        while self.match_any(&["+", "-"]) {
            let operator = self.previous();
            let right = self.factor()?;
            expr = Expr::Binary {
                left: Arc::new(expr),
                operator,
                right: Arc::new(right),
            };
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.unary()?;

        while self.match_any(&["*", "/"]) {
            let operator = self.previous();
            let right = self.unary()?;
            expr = Expr::Binary {
                left: Arc::new(expr),
                operator,
                right: Arc::new(right),
            };
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, RuntimeError> {
        if self.match_any(&["-", "!"]) {
            let operator = self.previous();
            let operand = Arc::new(self.unary()?);
            Ok(Expr::Unary { operator, operand })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, RuntimeError> {
        let mut expr = self.primary()?;

        loop {
            if self.match_token("(") {
                expr = self.finish_call(expr)?;
            } else if self.match_token("[") {
                let index = Arc::new(self.expression()?);
                self.consume("]", "Expected ']' after array index")?;
                expr = Expr::Index {
                    array: Arc::new(expr),
                    index,
                };
            } else if self.match_token(".") {
                let name = self.consume_identifier("Expected property name after '.'")?;
                expr = Expr::Binary {
                    left: Arc::new(expr),
                    operator: ".".to_string(),
                    right: Arc::new(Expr::Identifier(name)),
                };
            } else {
                break;
            }
        }

        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, RuntimeError> {
        let mut arguments = Vec::new();
        if !self.check(")") {
            loop {
                arguments.push(Arc::new(self.expression()?));
                if !self.match_token(",") {
                    break;
                }
            }
        }
        self.consume(")", "Expected ')' after arguments")?;

        Ok(Expr::Call {
            function: Arc::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, RuntimeError> {
        if self.match_token("false") {
            Ok(Expr::Boolean(false))
        } else if self.match_token("true") {
            Ok(Expr::Boolean(true))
        } else if self.match_token("null") {
            Ok(Expr::Object(vec![]))
        } else if self.is_number() {
            let value = self.current_token().parse::<f64>().unwrap();
            self.advance();
            Ok(Expr::Float(value))
        } else if self.is_string() {
            let value = self.current_token()[1..self.current_token().len()-1].to_string();
            self.advance();
            Ok(Expr::String(value))
        } else if self.is_identifier() {
            let name = self.current_token();
            self.advance();
            Ok(Expr::Identifier(name))
        } else if self.match_token("[") {
            let mut elements = Vec::new();
            if !self.check("]") {
                loop {
                    elements.push(Arc::new(self.expression()?));
                    if !self.match_token(",") {
                        break;
                    }
                }
            }
            self.consume("]", "Expected ']' after array elements")?;
            Ok(Expr::Array(elements))
        } else if self.match_token("{") {
            let mut fields = Vec::new();
            if !self.check("}") {
                loop {
                    let name = self.consume_string("Expected field name")?;
                    self.consume(":", "Expected ':' after field name")?;
                    let value = Arc::new(self.expression()?);
                    fields.push((name, value));
                    if !self.match_token(",") {
                        break;
                    }
                }
            }
            self.consume("}", "Expected '}' after object fields")?;
            Ok(Expr::Object(fields))
        } else {
            Err(RuntimeError::ParseError("Expected expression.".to_string()))
        }
    }

    // Helper methods for token handling
    fn match_token(&mut self, token: &str) -> bool {
        if self.check(token) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn match_any(&mut self, tokens: &[&str]) -> bool {
        for token in tokens {
            if self.check(token) {
                self.advance();
                return true;
            }
        }
        false
    }

    fn check(&self, token: &str) -> bool {
        if self.is_at_end() {
            false
        } else {
            self.current_token() == token
        }
    }

    fn advance(&mut self) -> String {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn current_token(&self) -> String {
        self.tokens[self.current].clone()
    }

    fn previous(&self) -> String {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token: &str, message: &str) -> Result<String, RuntimeError> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(RuntimeError::ParseError(message.to_string()))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, RuntimeError> {
        if self.is_identifier() {
            Ok(self.advance())
        } else {
            Err(RuntimeError::ParseError(message.to_string()))
        }
    }

    fn consume_string(&mut self, message: &str) -> Result<String, RuntimeError> {
        if self.is_string() {
            let value = self.current_token()[1..self.current_token().len()-1].to_string();
            self.advance();
            Ok(value)
        } else {
            Err(RuntimeError::ParseError(message.to_string()))
        }
    }

    fn is_number(&self) -> bool {
        !self.is_at_end() && self.current_token().parse::<f64>().is_ok()
    }

    fn is_string(&self) -> bool {
        !self.is_at_end() && self.current_token().starts_with('"') && self.current_token().ends_with('"')
    }

    fn is_identifier(&self) -> bool {
        !self.is_at_end() && self.current_token().chars().all(|c| c.is_alphanumeric() || c == '_')
    }
} 