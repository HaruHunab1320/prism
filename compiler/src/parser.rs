use crate::ast::{Expr, Stmt};
use crate::lexer::Token;
use crate::types::Value;

pub struct Parser {
    tokens: Vec<Token>,
    current: usize,
    source: String,
    token_starts: Vec<usize>,
    token_ends: Vec<usize>,
}

impl Parser {
    pub fn new(source: String, tokens: Vec<Token>, token_starts: Vec<usize>, token_ends: Vec<usize>) -> Self {
        Self {
            tokens,
            current: 0,
            source,
            token_starts,
            token_ends,
        }
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, Box<dyn std::error::Error>> {
        let mut statements = Vec::new();
        while !self.is_at_end() {
            statements.push(self.statement()?);
        }
        Ok(statements)
    }

    fn statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        match self.peek() {
            Some(Token::Let) => self.let_statement(),
            Some(Token::Context) => {
                if self.peek_next() == Some(&Token::Transition) {
                    self.context_transition_statement()
                } else {
                    self.context_statement()
                }
            },
            Some(Token::Verify) => self.verify_statement(),
            Some(Token::Try) => {
                if self.peek_next() == Some(&Token::Confidence) {
                    self.try_confidence_statement()
                } else {
                    Err("Expected 'confidence' after 'try'".into())
                }
            },
            Some(Token::Uncertain) => {
                self.advance();  // consume 'uncertain'
                self.uncertain_if_statement()
            },
            Some(Token::Match) => self.match_statement(),
            _ => {
                let expr = self.expression()?;
                self.consume(Token::Semicolon, "Expected ';' after expression")?;
                Ok(Stmt::Expression(expr))
            }
        }
    }

    fn let_statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        self.consume(Token::Let, "Expected 'let'")?;
        let name = match self.advance() {
            Some(Token::Identifier) => self.previous_token_str(),
            _ => return Err("Expected identifier after 'let'".into()),
        };

        self.consume(Token::Equal, "Expected '=' after variable name")?;
        let initializer = self.expression()?;
        self.consume(Token::Semicolon, "Expected ';' after variable declaration")?;

        Ok(Stmt::Let(name, Some(initializer)))
    }

    fn context_statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        self.consume(Token::Context, "Expected 'context'")?;
        let name = match self.advance() {
            Some(Token::String) => {
                let name = self.previous_token_str();
                name[1..name.len()-1].to_string()  // Remove quotes
            },
            _ => return Err("Expected context name".into()),
        };

        let body = Box::new(self.block_statement()?);
        Ok(Stmt::Context(name, body))
    }

    fn verify_statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        self.consume(Token::Verify, "Expected 'verify'")?;
        self.consume(Token::LeftBracket, "Expected '['")?;

        let mut sources = Vec::new();
        while !self.check(&Token::RightBracket) {
            match self.advance() {
                Some(Token::String) => {
                    let source = self.previous_token_str();
                    sources.push(source[1..source.len()-1].to_string());
                },
                _ => return Err("Expected source string".into()),
            }

            if !self.match_token(&[Token::Comma]) {
                break;
            }
        }

        self.consume(Token::RightBracket, "Expected ']'")?;
        let body = Box::new(self.block_statement()?);
        Ok(Stmt::Verify(sources, body))
    }

    fn try_confidence_statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        self.consume(Token::Try, "Expected 'try'")?;
        self.consume(Token::Confidence, "Expected 'confidence'")?;
        
        let body = Box::new(self.block_statement()?);
        
        self.consume(Token::Below, "Expected 'below'")?;
        self.consume(Token::Threshold, "Expected 'threshold'")?;
        
        let threshold = match self.advance() {
            Some(Token::Number) => {
                let num_str = self.previous_token_str();
                num_str.parse::<f64>()
                    .map_err(|_| "Invalid threshold value")?
            },
            _ => return Err("Expected number for threshold".into()),
        };
        
        let below_threshold = Box::new(self.block_statement()?);
        
        let uncertain = if self.match_token(&[Token::Uncertain]) {
            Some(Box::new(self.block_statement()?))
        } else {
            None
        };
        
        Ok(Stmt::TryConfidence {
            body,
            below_threshold,
            uncertain: uncertain.unwrap_or_else(|| Box::new(Stmt::Block(vec![]))),
            threshold,
        })
    }

    fn uncertain_if_statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        self.consume(Token::If, "Expected 'if' after 'uncertain'")?;
        
        self.consume(Token::LeftParen, "Expected '(' after 'if'")?;
        let condition = self.expression()?;
        self.consume(Token::RightParen, "Expected ')' after condition")?;
        
        let then_branch = Box::new(self.block_statement()?);
        
        let medium_branch = if self.match_token(&[Token::Medium]) {
            Some(Box::new(self.block_statement()?))
        } else {
            None
        };
        
        let else_branch = if self.match_token(&[Token::Low]) {
            Some(Box::new(self.block_statement()?))
        } else {
            None
        };

        Ok(Stmt::UncertainIf(condition, then_branch, medium_branch, else_branch))
    }

    fn match_statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        self.consume(Token::Match, "Expected 'match'")?;
        let value = self.expression()?;
        
        self.consume(Token::LeftBrace, "Expected '{' after match expression")?;
        
        let mut patterns = Vec::new();
        let mut bodies = Vec::new();
        
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            let pattern = self.expression()?;
            self.consume(Token::Arrow, "Expected '=>' after pattern")?;
            
            let body = if self.check(&Token::LeftBrace) {
                self.block_statement()?
            } else {
                let expr = self.expression()?;
                self.consume(Token::Semicolon, "Expected ';' after match arm")?;
                Stmt::Expression(expr)
            };
            
            patterns.push(pattern);
            bodies.push(body);
            
            if !self.match_token(&[Token::Comma]) && !self.check(&Token::RightBrace) {
                return Err("Expected ',' or '}' after match arm".into());
            }
        }
        
        self.consume(Token::RightBrace, "Expected '}' after match arms")?;
        
        Ok(Stmt::Match {
            value: Box::new(value),
            patterns,
            bodies,
        })
    }

    fn block_statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        self.consume(Token::LeftBrace, "Expected '{'")?;
        
        let mut statements = Vec::new();
        while !self.check(&Token::RightBrace) && !self.is_at_end() {
            statements.push(self.statement()?);
        }
        
        self.consume(Token::RightBrace, "Expected '}'")?;
        Ok(Stmt::Block(statements))
    }

    fn expression(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut expr = self.equality()?;

        if self.match_token(&[Token::SemanticMatch]) {
            let right = self.equality()?;
            expr = Expr::SemanticMatch(Box::new(expr), Box::new(right));
        } else if self.match_token(&[Token::ConfidenceFlow]) {
            let right = self.equality()?;
            expr = Expr::ConfidenceFlow(Box::new(expr), Box::new(right));
        } else if self.match_token(&[Token::ConfidenceAssign]) {
            let confidence = match self.advance() {
                Some(Token::Number) => {
                    let num_str = self.previous_token_str();
                    num_str.parse::<f64>()
                        .map_err(|_| "Invalid confidence value")?
                },
                _ => return Err("Expected confidence value after '~'".into()),
            };
            expr = Expr::ConfidenceAssign(Box::new(expr), confidence);
        } else if self.match_token(&[Token::In]) {
            self.consume(Token::Context, "Expected 'context' after 'in'")?;
            let context = match self.advance() {
                Some(Token::String) => {
                    let name = self.previous_token_str();
                    name[1..name.len()-1].to_string()  // Remove quotes
                },
                _ => return Err("Expected context name".into()),
            };
            expr = Expr::InContext(context, Box::new(expr));
        }

        Ok(expr)
    }

    fn equality(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut expr = self.comparison()?;

        while self.match_token(&[Token::BangEqual, Token::EqualEqual]) {
            let operator = self.previous_token_str();
            let right = self.comparison()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn comparison(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut expr = self.term()?;

        while self.match_token(&[Token::Greater, Token::GreaterEqual, Token::Less, Token::LessEqual]) {
            let operator = self.previous_token_str();
            let right = self.term()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn term(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut expr = self.factor()?;

        while self.match_token(&[Token::Plus, Token::Minus]) {
            let operator = self.previous_token_str();
            let right = self.factor()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn factor(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let mut expr = self.unary()?;

        while self.match_token(&[Token::Star, Token::Slash]) {
            let operator = self.previous_token_str();
            let right = self.unary()?;
            expr = Expr::Binary(Box::new(expr), operator, Box::new(right));
        }

        Ok(expr)
    }

    fn unary(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        if self.match_token(&[Token::Bang, Token::Minus]) {
            let operator = self.previous_token_str();
            let right = self.unary()?;
            return Ok(Expr::Unary(operator, Box::new(right)));
        }

        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        if self.match_token(&[Token::Tensor]) {
            return self.tensor_expression();
        }
        
        match self.advance() {
            Some(Token::Number) => self.number(),
            Some(Token::String) => self.string(),
            Some(Token::Identifier) => {
                let name = self.previous_token_str();
                if self.match_token(&[Token::Dot]) {
                    let method = match self.advance() {
                        Some(Token::Identifier) => self.previous_token_str(),
                        _ => return Err("Expected method name after '.'".into()),
                    };
                    
                    // Handle tensor methods
                    match method.as_str() {
                        "cosine_similarity" | "dot_product" | "normalize" | "transpose" | "reshape" => {
                            self.consume(Token::LeftParen, "Expected '(' after tensor method name")?;
                            let mut arguments = Vec::new();
                            
                            if !self.check(&Token::RightParen) {
                                loop {
                                    arguments.push(self.expression()?);
                                    if !self.match_token(&[Token::Comma]) {
                                        break;
                                    }
                                }
                            }
                            
                            self.consume(Token::RightParen, "Expected ')' after arguments")?;
                            Ok(Expr::Call(Box::new(Expr::Get(Box::new(Expr::Variable(name)), method)), arguments))
                        },
                        _ => {
                            self.consume(Token::LeftParen, "Expected '(' after method name")?;
                            let mut arguments = Vec::new();
                            
                            if !self.check(&Token::RightParen) {
                                loop {
                                    arguments.push(self.expression()?);
                                    if !self.match_token(&[Token::Comma]) {
                                        break;
                                    }
                                }
                            }
                            
                            self.consume(Token::RightParen, "Expected ')' after arguments")?;
                            Ok(Expr::Call(Box::new(Expr::Get(Box::new(Expr::Variable(name)), method)), arguments))
                        }
                    }
                } else {
                    Ok(Expr::Variable(name))
                }
            },
            Some(Token::LeftParen) => {
                let expr = self.expression()?;
                self.consume(Token::RightParen, "Expected ')' after expression")?;
                Ok(Expr::Grouping(Box::new(expr)))
            },
            _ => Err("Expected expression".into()),
        }
    }

    fn tensor_expression(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        self.consume(Token::LeftBracket, "Expected '[' after 'tensor'")?;
        
        let mut values = Vec::new();
        while !self.check(&Token::RightBracket) {
            let value = self.expression()?;
            values.push(value);
            
            if !self.match_token(&[Token::Comma]) {
                break;
            }
        }
        
        self.consume(Token::RightBracket, "Expected ']'")?;
        
        // Parse shape if provided
        let shape = if self.match_token(&[Token::Comma]) {
            self.consume(Token::LeftBracket, "Expected '['")?;
            let mut dims = Vec::new();
            while !self.check(&Token::RightBracket) {
                match self.advance() {
                    Some(Token::Number) => {
                        let num_str = self.previous_token_str();
                        let dim = num_str.parse::<usize>()
                            .map_err(|_| "Invalid tensor dimension")?;
                        dims.push(dim);
                    },
                    _ => return Err("Expected number for tensor dimension".into()),
                }
                
                if !self.match_token(&[Token::Comma]) {
                    break;
                }
            }
            self.consume(Token::RightBracket, "Expected ']'")?;
            Some(dims)
        } else {
            None
        };
        
        Ok(Expr::Tensor { values: Box::new(values), shape })
    }

    fn number(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let token = self.previous_token_str();
        let value = if token.contains('.') {
            token.parse::<f64>().map_err(|_| "Invalid float literal")?
        } else {
            token.parse::<i64>().map_err(|_| "Invalid integer literal")? as f64
        };
        Ok(Expr::Literal(Value::Float(value)))
    }

    fn string(&mut self) -> Result<Expr, Box<dyn std::error::Error>> {
        let token = self.previous_token_str();
        let value = token[1..token.len()-1].to_string();  // Remove quotes
        Ok(Expr::Literal(Value::String(value)))
    }

    fn peek(&mut self) -> Option<&Token> {
        self.tokens.get(self.current)
    }

    fn peek_next(&mut self) -> Option<&Token> {
        self.tokens.get(self.current + 1)
    }

    fn advance(&mut self) -> Option<Token> {
        if !self.is_at_end() {
            self.current += 1;
        }
        self.previous()
    }

    fn previous(&mut self) -> Option<Token> {
        if self.current > 0 {
            self.tokens.get(self.current - 1).cloned()
        } else {
            None
        }
    }

    fn previous_token_str(&mut self) -> String {
        if self.current > 0 {
            let start = self.token_starts[self.current - 1];
            let end = self.token_ends[self.current - 1];
            self.source[start..end].to_string()
        } else {
            String::new()
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

    fn check(&mut self, token: &Token) -> bool {
        if let Some(current) = self.peek() {
            current == token
        } else {
            false
        }
    }

    fn consume(&mut self, token: Token, message: &str) -> Result<(), Box<dyn std::error::Error>> {
        if self.check(&token) {
            self.advance();
            Ok(())
        } else {
            Err(message.into())
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.tokens.len()
    }

    fn context_transition_statement(&mut self) -> Result<Stmt, Box<dyn std::error::Error>> {
        self.consume(Token::Context, "Expected 'context'")?;
        self.consume(Token::Transition, "Expected 'transition'")?;
        
        let from_context = match self.advance() {
            Some(Token::String) => {
                let name = self.previous_token_str();
                name[1..name.len()-1].to_string()  // Remove quotes
            },
            _ => return Err("Expected source context name".into()),
        };
        
        self.consume(Token::To, "Expected 'to'")?;
        
        let to_context = match self.advance() {
            Some(Token::String) => {
                let name = self.previous_token_str();
                name[1..name.len()-1].to_string()  // Remove quotes
            },
            _ => return Err("Expected target context name".into()),
        };
        
        let confidence = if self.match_token(&[Token::With]) {
            self.consume(Token::Confidence, "Expected 'confidence'")?;
            match self.advance() {
                Some(Token::Number) => {
                    let num_str = self.previous_token_str();
                    Some(num_str.parse::<f64>()
                        .map_err(|_| "Invalid confidence value")?)
                },
                _ => return Err("Expected confidence value".into()),
            }
        } else {
            None
        };
        
        let body = Box::new(self.block_statement()?);
        
        Ok(Stmt::ContextTransition {
            from_context,
            to_context,
            confidence,
            body,
        })
    }
} 