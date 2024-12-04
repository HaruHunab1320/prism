use crate::ast::{Expr, Stmt};
use crate::lexer::Token;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("{message}")]
pub struct ParseError {
    pub message: String,
    pub position: usize,
}

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

    pub fn parse(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        Ok(statements)
    }

    fn declaration(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[Token::Let]) {
            self.let_declaration()
        } else if self.match_token(&[Token::Function]) {
            self.function()
        } else if self.match_token(&[Token::Context]) {
            self.context_declaration()
        } else if self.match_token(&[Token::Verify]) {
            self.verify_declaration()
        } else {
            self.statement()
        }
    }

    fn let_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("Expected variable name")?;
        self.consume(&Token::Equal, "Expected '=' after variable name")?;
        let initializer = self.expression()?;
        self.consume(&Token::Semicolon, "Expected ';' after variable declaration")?;
        Ok(Stmt::Let {
            name,
            initializer,
        })
    }

    fn context_declaration(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_string("Expected context name")?;
        self.consume(&Token::LBrace, "Expected '{' after context name")?;
        let mut statements = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&Token::RBrace, "Expected '}' after context block")?;
        Ok(Stmt::Block(statements))
    }

    fn verify_declaration(&mut self) -> Result<Stmt, ParseError> {
        let condition = self.expression()?;
        self.consume(&Token::LBrace, "Expected '{' after verify condition")?;
        let mut statements = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&Token::RBrace, "Expected '}' after verify block")?;
        Ok(Stmt::Block(statements))
    }

    fn function(&mut self) -> Result<Stmt, ParseError> {
        let name = self.consume_identifier("Expected function name")?;
        self.consume(&Token::LParen, "Expected '(' after function name")?;
        let mut params = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                params.push(self.consume_identifier("Expected parameter name")?);
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
        self.consume(&Token::RParen, "Expected ')' after parameters")?;
        self.consume(&Token::LBrace, "Expected '{' before function body")?;
        let body = Box::new(Stmt::Block(self.block()?));
        Ok(Stmt::Function {
            name,
            params,
            body,
        })
    }

    fn statement(&mut self) -> Result<Stmt, ParseError> {
        if self.match_token(&[Token::If]) {
            self.if_statement()
        } else if self.match_token(&[Token::While]) {
            self.while_statement()
        } else if self.match_token(&[Token::Break]) {
            self.consume(&Token::Semicolon, "Expected ';' after 'break'")?;
            Ok(Stmt::Break)
        } else if self.match_token(&[Token::Continue]) {
            self.consume(&Token::Semicolon, "Expected ';' after 'continue'")?;
            Ok(Stmt::Continue)
        } else if self.match_token(&[Token::Return]) {
            let value = self.expression()?;
            self.consume(&Token::Semicolon, "Expected ';' after return value")?;
            Ok(Stmt::Return(value))
        } else if self.match_token(&[Token::Try]) {
            self.try_catch()
        } else if self.match_token(&[Token::Throw]) {
            let value = self.expression()?;
            self.consume(&Token::Semicolon, "Expected ';' after throw value")?;
            Ok(Stmt::Throw(value))
        } else if self.match_token(&[Token::LBrace]) {
            Ok(Stmt::Block(self.block()?))
        } else {
            let expr = self.expression()?;
            self.consume(&Token::Semicolon, "Expected ';' after expression")?;
            Ok(Stmt::Expression(expr))
        }
    }

    fn if_statement(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&Token::LParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(&Token::RParen, "Expected ')' after if condition")?;
        let then_branch = Box::new(self.statement()?);
        let else_branch = if self.match_token(&[Token::Else]) {
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
        self.consume(&Token::LParen, "Expected '(' after 'while'")?;
        let condition = match self.expression() {
            Ok(expr) => expr,
            Err(e) => {
                return Err(ParseError {
                    message: format!("Error parsing while condition: {}", e.message),
                    position: e.position,
                });
            }
        };
        let current_pos = self.current;
        let current_token = if current_pos < self.tokens.len() {
            Some(self.tokens[current_pos].clone())
        } else {
            None
        };
        let next_token = if current_pos + 1 < self.tokens.len() {
            Some(self.tokens[current_pos + 1].clone())
        } else {
            None
        };
        match self.consume(&Token::RParen, "Expected ')' after while condition") {
            Ok(_) => (),
            Err(e) => {
                return Err(ParseError {
                    message: format!("Missing closing parenthesis: {} (current token: {:?}, next token: {:?})", e.message, current_token, next_token),
                    position: e.position,
                });
            }
        }
        let body = Box::new(self.statement()?);
        Ok(Stmt::While {
            condition,
            body,
        })
    }

    fn try_catch(&mut self) -> Result<Stmt, ParseError> {
        self.consume(&Token::LBrace, "Expected '{' after 'try'")?;
        let try_block = Box::new(Stmt::Block(self.block()?));
        self.consume(&Token::Catch, "Expected 'catch' after try block")?;
        self.consume(&Token::LParen, "Expected '(' after 'catch'")?;
        let catch_variable = self.consume_identifier("Expected catch variable name")?;
        self.consume(&Token::RParen, "Expected ')' after catch variable")?;
        self.consume(&Token::LBrace, "Expected '{' before catch block")?;
        let catch_block = Box::new(Stmt::Block(self.block()?));
        Ok(Stmt::TryCatch {
            try_block,
            catch_variable,
            catch_block,
        })
    }

    fn block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while !self.check(&Token::RBrace) && !self.is_at_end() {
            statements.push(self.declaration()?);
        }
        self.consume(&Token::RBrace, "Expected '}' after block")?;
        Ok(statements)
    }

    fn expression(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.confidence_flow()?;
        while self.match_token(&[Token::Dot]) {
            let name = self.consume_identifier("Expected property name after '.'")?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: ".".to_string(),
                right: Box::new(Expr::Identifier(name)),
            };
        }
        Ok(expr)
    }

    fn confidence_flow(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.assignment()?;
        while self.match_token(&[Token::Tilde]) {
            self.consume(&Token::Greater, "Expected '>' after '~'")?;
            let confidence = self.expression()?;
            expr = Expr::Binary {
                left: Box::new(expr),
                operator: "~>".to_string(),
                right: Box::new(confidence),
            };
        }
        Ok(expr)
    }

    fn assignment(&mut self) -> Result<Expr, ParseError> {
        let expr = self.equality()?;
        if self.match_token(&[Token::Equal]) {
            let value = self.assignment()?;
            match expr {
                Expr::Identifier(name) => {
                    return Ok(Expr::Binary {
                        left: Box::new(Expr::Identifier(name)),
                        operator: "=".to_string(),
                        right: Box::new(value),
                    });
                }
                _ => {
                    return Err(ParseError {
                        message: "Invalid assignment target".to_string(),
                        position: self.current,
                    });
                }
            }
        }
        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.comparison()?;
        while self.match_token(&[Token::EqualEqual, Token::BangEqual]) {
            let operator = self.previous().to_string();
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
        while self.match_token(&[Token::Less, Token::LessEqual, Token::Greater, Token::GreaterEqual]) {
            let operator = self.previous().to_string();
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
        while self.match_token(&[Token::Plus, Token::Minus]) {
            let operator = self.previous().to_string();
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
        while self.match_token(&[Token::Star, Token::Slash]) {
            let operator = self.previous().to_string();
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
        if self.match_token(&[Token::Minus, Token::Bang]) {
            let operator = self.previous().to_string();
            let operand = Box::new(self.unary()?);
            Ok(Expr::Unary {
                operator,
                operand,
            })
        } else {
            self.call()
        }
    }

    fn call(&mut self) -> Result<Expr, ParseError> {
        let mut expr = self.primary()?;
        loop {
            if self.match_token(&[Token::LParen]) {
                expr = self.finish_call(expr)?;
            } else if self.match_token(&[Token::LBracket]) {
                let index = self.expression()?;
                self.consume(&Token::RBracket, "Expected ']' after array index")?;
                expr = Expr::Index {
                    array: Box::new(expr),
                    index: Box::new(index),
                };
            } else if self.match_token(&[Token::Dot]) {
                let name = self.consume_identifier("Expected property name after '.'")?;
                expr = Expr::Binary {
                    left: Box::new(expr),
                    operator: ".".to_string(),
                    right: Box::new(Expr::Identifier(name)),
                };
            } else {
                break;
            }
        }
        Ok(expr)
    }

    fn finish_call(&mut self, callee: Expr) -> Result<Expr, ParseError> {
        let mut arguments = Vec::new();
        if !self.check(&Token::RParen) {
            loop {
                arguments.push(self.expression()?);
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
        }
        self.consume(&Token::RParen, "Expected ')' after arguments")?;
        Ok(Expr::Call {
            function: Box::new(callee),
            arguments,
        })
    }

    fn primary(&mut self) -> Result<Expr, ParseError> {
        if self.match_token(&[Token::Float(0.0)]) {
            if let Token::Float(value) = self.previous() {
                Ok(Expr::Float(value))
            } else {
                unreachable!()
            }
        } else if self.match_token(&[Token::String("".to_string())]) {
            if let Token::String(value) = self.previous() {
                Ok(Expr::String(value))
            } else {
                unreachable!()
            }
        } else if self.match_token(&[Token::Boolean(false)]) {
            if let Token::Boolean(value) = self.previous() {
                Ok(Expr::Boolean(value))
            } else {
                unreachable!()
            }
        } else if self.match_token(&[Token::Identifier("".to_string())]) {
            if let Token::Identifier(name) = self.previous() {
                Ok(Expr::Identifier(name))
            } else {
                unreachable!()
            }
        } else if self.match_token(&[Token::LBracket]) {
            let mut elements = Vec::new();
            if !self.check(&Token::RBracket) {
                loop {
                    elements.push(self.expression()?);
                    if !self.match_token(&[Token::Comma]) {
                        break;
                    }
                }
            }
            self.consume(&Token::RBracket, "Expected ']' after array elements")?;
            Ok(Expr::Array(elements))
        } else if self.match_token(&[Token::LBrace]) {
            let mut fields = Vec::new();
            if !self.check(&Token::RBrace) {
                loop {
                    let name = self.consume_identifier("Expected field name")?;
                    self.consume(&Token::Colon, "Expected ':' after field name")?;
                    let value = self.expression()?;
                    fields.push((name, value));
                    if !self.match_token(&[Token::Comma]) {
                        break;
                    }
                }
            }
            self.consume(&Token::RBrace, "Expected '}' after object fields")?;
            Ok(Expr::Object(fields))
        } else if self.match_token(&[Token::LParen]) {
            let expr = self.expression()?;
            self.consume(&Token::RParen, "Expected ')' after expression")?;
            Ok(expr)
        } else {
            Err(ParseError {
                message: format!("Expected expression, got {:?}", self.peek()),
                position: self.current,
            })
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
            match self.current_token() {
                Some(t) => std::mem::discriminant(t) == std::mem::discriminant(token),
                None => false,
            }
        }
    }

    fn advance(&mut self) -> Token {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn peek(&self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            Some(&self.tokens[self.current])
        } else {
            None
        }
    }

    fn previous(&self) -> Token {
        self.tokens[self.current - 1].clone()
    }

    fn consume(&mut self, token: &Token, message: &str) -> Result<Token, ParseError> {
        if self.check(token) {
            Ok(self.advance())
        } else {
            Err(ParseError {
                message: message.to_string(),
                position: self.current,
            })
        }
    }

    fn consume_identifier(&mut self, message: &str) -> Result<String, ParseError> {
        if let Some(Token::Identifier(name)) = self.current_token().cloned() {
            self.advance();
            Ok(name)
        } else {
            Err(ParseError {
                message: message.to_string(),
                position: self.current,
            })
        }
    }

    fn consume_string(&mut self, message: &str) -> Result<String, ParseError> {
        if let Some(Token::String(s)) = self.current_token().cloned() {
            self.advance();
            Ok(s)
        } else {
            Err(ParseError {
                message: message.to_string(),
                position: self.current,
            })
        }
    }

    fn current_token(&self) -> Option<&Token> {
        if self.current < self.tokens.len() {
            Some(&self.tokens[self.current])
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_expression() {
        let tokens = vec![
            Token::Float(1.0),
            Token::Plus,
            Token::Float(2.0),
            Token::Semicolon,
        ];
        let mut parser = Parser::new(tokens);
        let stmt = parser.parse().unwrap();
        assert!(matches!(
            stmt[0],
            Stmt::Expression(Expr::Binary {
                left: _,
                operator: _,
                right: _,
            })
        ));
    }

    #[test]
    fn test_parse_let() {
        let tokens = vec![
            Token::Let,
            Token::Identifier("x".to_string()),
            Token::Equal,
            Token::Float(42.0),
            Token::Semicolon,
        ];
        let mut parser = Parser::new(tokens);
        let stmt = parser.parse().unwrap();
        assert!(matches!(
            stmt[0],
            Stmt::Let {
                name: _,
                initializer: _,
            }
        ));
    }

    #[test]
    fn test_parse_function() {
        let tokens = vec![
            Token::Function,
            Token::Identifier("test".to_string()),
            Token::LParen,
            Token::Identifier("x".to_string()),
            Token::RParen,
            Token::LBrace,
            Token::Return,
            Token::Float(1.0),
            Token::Semicolon,
            Token::RBrace,
        ];
        let mut parser = Parser::new(tokens);
        let stmt = parser.parse().unwrap();
        assert!(matches!(
            stmt[0],
            Stmt::Function {
                name: _,
                params: _,
                body: _,
            }
        ));
    }
} 