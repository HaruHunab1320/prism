use crate::ast::{Expr, Stmt};
use crate::error::{PrismError, Result};
use crate::token::{Token, TokenKind};
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

    pub fn parse_expression(&mut self) -> Result<Expr> {
        self.expression()
    }

    fn declaration(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenKind::Let]) {
            self.let_declaration()
        } else if self.match_token(&[TokenKind::Fun]) {
            self.function_declaration()
        } else if self.match_token(&[TokenKind::Export]) {
            self.export_declaration()
        } else if self.match_token(&[TokenKind::Import]) {
            self.import_declaration()
        } else if self.match_token(&[TokenKind::Module]) {
            self.module_declaration()
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

        self.consume(&TokenKind::Semicolon, "Expected ';' after variable declaration.")?;
        Ok(Stmt::Let(name, initializer))
    }

    fn function_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier("Expected function name.")?;
        self.consume(&TokenKind::LeftParen, "Expected '(' after function name.")?;
        
        let mut parameters = Vec::new();
        if !self.check(&TokenKind::RightParen) {
            loop {
                parameters.push(self.consume_identifier("Expected parameter name.")?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(&TokenKind::RightParen, "Expected ')' after parameters.")?;
        
        let is_async = self.match_token(&[TokenKind::Async]);
        
        let mut confidence = None;
        if self.match_token(&[TokenKind::Confidence]) {
            confidence = Some(self.consume_number("Expected confidence value.")?);
        }
        
        self.consume(&TokenKind::LeftBrace, "Expected '{' before function body.")?;
        let body = Box::new(self.block()?);
        
        Ok(Stmt::Function {
            name,
            params: parameters,
            body,
            is_async,
            confidence,
        })
    }

    fn export_declaration(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenKind::Let]) {
            let stmt = self.let_declaration()?;
            if let Stmt::Let(name, init) = stmt {
                let name_clone = name.clone();
                Ok(Stmt::Export(name, Box::new(Stmt::Let(name_clone, init))))
            } else {
                Err(Box::new(PrismError::Parse(
                    "Expected let declaration after export.".to_string()
                )))
            }
        } else if self.match_token(&[TokenKind::Fun]) {
            let stmt = self.function_declaration()?;
            if let Stmt::Function { name, .. } = &stmt {
                let name = name.clone();
                Ok(Stmt::Export(name, Box::new(stmt)))
            } else {
                Err(Box::new(PrismError::Parse(
                    "Expected function declaration after export.".to_string()
                )))
            }
        } else {
            Err(Box::new(PrismError::Parse(
                "Expected declaration after export.".to_string()
            )))
        }
    }

    fn import_declaration(&mut self) -> Result<Stmt> {
        let mut imports = Vec::new();
        
        if self.match_token(&[TokenKind::LeftBrace]) {
            loop {
                let name = self.consume_identifier("Expected import name.")?;
                let alias = if self.match_token(&[TokenKind::As]) {
                    Some(self.consume_identifier("Expected alias name.")?)
                } else {
                    None
                };
                imports.push((name, alias));
                
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
            self.consume(&TokenKind::RightBrace, "Expected '}' after imports.")?;
        } else {
            let name = self.consume_identifier("Expected import name.")?;
            imports.push((name, None));
        }
        
        self.consume(&TokenKind::From, "Expected 'from' after imports.")?;
        let module = self.consume_string("Expected module path.")?;
        
        let mut confidence = None;
        if self.match_token(&[TokenKind::Confidence]) {
            confidence = Some(self.consume_number("Expected confidence value.")?);
        }
        
        self.consume(&TokenKind::Semicolon, "Expected ';' after import.")?;
        
        Ok(Stmt::Import {
            module,
            imports,
            confidence,
        })
    }

    fn module_declaration(&mut self) -> Result<Stmt> {
        let name = self.consume_identifier("Expected module name.")?;
        
        let mut confidence = None;
        if self.match_token(&[TokenKind::Confidence]) {
            confidence = Some(self.consume_number("Expected confidence value.")?);
        }
        
        self.consume(&TokenKind::LeftBrace, "Expected '{' before module body.")?;
        
        let mut body = Vec::new();
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            body.push(self.declaration()?);
        }
        
        self.consume(&TokenKind::RightBrace, "Expected '}' after module body.")?;
        
        Ok(Stmt::Module {
            name,
            body,
            confidence,
        })
    }

    fn statement(&mut self) -> Result<Stmt> {
        if self.match_token(&[TokenKind::If]) {
            self.if_statement()
        } else if self.match_token(&[TokenKind::While]) {
            self.while_statement()
        } else if self.match_token(&[TokenKind::Return]) {
            self.return_statement()
        } else if self.match_token(&[TokenKind::LeftBrace]) {
            Ok(Stmt::Block(self.block_statements()?))
        } else {
            self.expression_statement()
        }
    }

    fn if_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'if'.")?;
        let condition = Box::new(self.expression()?);
        self.consume(&TokenKind::RightParen, "Expected ')' after if condition.")?;
        
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[TokenKind::Else]) {
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

    fn while_statement(&mut self) -> Result<Stmt> {
        self.consume(&TokenKind::LeftParen, "Expected '(' after 'while'.")?;
        let condition = Box::new(self.expression()?);
        self.consume(&TokenKind::RightParen, "Expected ')' after while condition.")?;
        
        let body = Box::new(self.statement()?);
        
        Ok(Stmt::While { condition, body })
    }

    fn return_statement(&mut self) -> Result<Stmt> {
        let value = if !self.check(&TokenKind::Semicolon) {
            Some(Box::new(self.expression()?))
        } else {
            None
        };
        
        self.consume(&TokenKind::Semicolon, "Expected ';' after return value.")?;
        Ok(Stmt::Return(value))
    }

    fn expression_statement(&mut self) -> Result<Stmt> {
        let expr = self.expression()?;
        self.consume(&TokenKind::Semicolon, "Expected ';' after expression.")?;
        Ok(Stmt::Expression(Box::new(expr)))
    }

    fn block(&mut self) -> Result<Stmt> {
        Ok(Stmt::Block(self.block_statements()?))
    }

    fn block_statements(&mut self) -> Result<Vec<Stmt>> {
        let mut statements = Vec::new();
        
        while !self.check(&TokenKind::RightBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        
        self.consume(&TokenKind::RightBrace, "Expected '}' after block.")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr> {
        self.assignment()
    }

    fn assignment(&mut self) -> Result<Expr> {
        let expr = self.or()?;
        
        if self.match_token(&[TokenKind::Equal]) {
            let value = Box::new(self.assignment()?);
            
            if let Expr::Variable(name) = expr {
                return Ok(Expr::Assign {
                    name,
                    value,
                });
            }
            
            return Err(Box::new(PrismError::Parse(format!(
                "Invalid assignment target at line {}.",
                self.previous().line
            ))));
        }
        
        Ok(expr)
    }

    fn or(&mut self) -> Result<Expr> {
        let mut expr = self.and()?;
        
        while self.match_token(&[TokenKind::Or]) {
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

    fn and(&mut self) -> Result<Expr> {
        let mut expr = self.equality()?;
        
        while self.match_token(&[TokenKind::And]) {
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

    fn equality(&mut self) -> Result<Expr> {
        let mut expr = self.comparison()?;
        
        while self.match_token(&[TokenKind::BangEqual, TokenKind::EqualEqual]) {
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

    fn comparison(&mut self) -> Result<Expr> {
        let mut expr = self.term()?;
        
        while self.match_token(&[
            TokenKind::Greater,
            TokenKind::GreaterEqual,
            TokenKind::Less,
            TokenKind::LessEqual,
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

    fn term(&mut self) -> Result<Expr> {
        let mut expr = self.factor()?;
        
        while self.match_token(&[TokenKind::Minus, TokenKind::Plus]) {
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

    fn factor(&mut self) -> Result<Expr> {
        let mut expr = self.unary()?;
        
        while self.match_token(&[TokenKind::Slash, TokenKind::Star]) {
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

    fn unary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenKind::Bang, TokenKind::Minus]) {
            let operator = self.previous().clone();
            let right = Box::new(self.unary()?);
            Ok(Expr::Unary { operator, right })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr> {
        let mut expr = self.primary()?;
        
        loop {
            if self.match_token(&[TokenKind::LeftParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[TokenKind::Dot]) {
                let name = self.consume_identifier("Expected property name after '.'.")?;
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

    fn finish_call(&mut self, callee: Expr) -> Result<Expr> {
        let mut arguments = Vec::new();
        
        if !self.check(&TokenKind::RightParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[TokenKind::Comma]) {
                    break;
                }
            }
        }
        
        self.consume(&TokenKind::RightParen, "Expected ')' after arguments.")?;
        
        Ok(Expr::Call {
            callee: Box::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr> {
        if self.match_token(&[TokenKind::False]) {
            Ok(Expr::Literal(Value::new(ValueKind::Boolean(false))))
        } else if self.match_token(&[TokenKind::True]) {
            Ok(Expr::Literal(Value::new(ValueKind::Boolean(true))))
        } else if self.match_token(&[TokenKind::Nil]) {
            Ok(Expr::Literal(Value::new(ValueKind::Nil)))
        } else {
            let token = self.peek().cloned();
            match token {
                Some(Token { kind: TokenKind::Number(n), .. }) => {
                    self.advance();
                    Ok(Expr::Literal(Value::new(ValueKind::Number(n))))
                }
                Some(Token { kind: TokenKind::String(s), .. }) => {
                    self.advance();
                    Ok(Expr::Literal(Value::new(ValueKind::String(s))))
                }
                Some(Token { kind: TokenKind::Identifier(name), .. }) => {
                    self.advance();
                    Ok(Expr::Variable(name))
                }
                Some(Token { kind: TokenKind::LeftParen, .. }) => {
                    self.advance();
                    let expr = self.expression()?;
                    self.consume(&TokenKind::RightParen, "Expected ')' after expression.")?;
                    Ok(Expr::Grouping(Box::new(expr)))
                }
                _ => Err(Box::new(PrismError::Parse(
                    "Expected expression.".to_string()
                ))),
            }
        }
    }

    fn consume(&mut self, kind: &TokenKind, message: &str) -> Result<()> {
        if self.check(kind) {
            self.advance();
            Ok(())
        } else {
            Err(Box::new(PrismError::Parse(message.to_string())))
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String> {
        let token = self.peek().cloned();
        if let Some(Token { kind: TokenKind::Identifier(name), .. }) = token {
            self.advance();
            Ok(name)
        } else {
            Err(Box::new(PrismError::Parse(message.to_string())))
        }
    }

    fn consume_string(&mut self, message: &str) -> Result<String> {
        let token = self.peek().cloned();
        if let Some(Token { kind: TokenKind::String(s), .. }) = token {
            self.advance();
            Ok(s)
        } else {
            Err(Box::new(PrismError::Parse(message.to_string())))
        }
    }

    fn consume_number(&mut self, message: &str) -> Result<f64> {
        let token = self.peek().cloned();
        if let Some(Token { kind: TokenKind::Number(n), .. }) = token {
            self.advance();
            Ok(n)
        } else {
            Err(Box::new(PrismError::Parse(message.to_string())))
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
        if let Some(token) = self.peek() {
            &token.kind == kind
        } else {
            false
        }
    }

    fn advance(&mut self) -> &Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        matches!(self.peek().map(|t| &t.kind), Some(TokenKind::EOF))
    }

    fn peek(&self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            Some(&self.tokens[self.current])
        } else {
            None
        }
    }

    fn previous(&self) -> &Token {
        &self.tokens[self.current - 1]
    }
}
