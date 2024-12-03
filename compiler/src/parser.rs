use crate::ast::{Expr, Stmt, BinaryOperator};
use crate::lexer::Token;
use std::iter::Peekable;

pub struct Parser<I>
where
    I: Iterator<Item = Token>,
{
    tokens: Peekable<I>,
}

#[derive(Debug)]
pub enum ParseError {
    UnexpectedToken(String),
    UnexpectedEOF,
    InvalidNumber(String),
}

impl<I> Parser<I>
where
    I: Iterator<Item = Token>,
{
    pub fn new(tokens: I) -> Self {
        Self {
            tokens: tokens.peekable(),
        }
    }

    pub fn parse_program(&mut self) -> Result<Vec<Stmt>, ParseError> {
        let mut statements = Vec::new();
        while self.tokens.peek().is_some() {
            statements.push(self.parse_statement()?);
        }
        Ok(statements)
    }

    fn parse_statement(&mut self) -> Result<Stmt, ParseError> {
        match self.tokens.peek() {
            Some(Token::Confidence) => self.parse_confidence_declaration(),
            Some(Token::Uncertain) => self.parse_uncertain_if(),
            Some(Token::In) => self.parse_context_block(),
            Some(Token::Verify) => self.parse_verification(),
            Some(Token::Identifier(_)) => {
                let id = self.parse_identifier()?;
                match self.tokens.peek() {
                    Some(Token::Assign) => {
                        self.tokens.next(); // consume =
                        let value = self.parse_expression()?;
                        Ok(Stmt::Assignment {
                            target: id,
                            value,
                        })
                    }
                    Some(Token::ConfidenceFlow) => {
                        self.tokens.next(); // consume ~>
                        let target = self.parse_expression()?;
                        Ok(Stmt::Expression(Expr::ConfidenceFlow {
                            source: Box::new(Expr::Identifier(id)),
                            target: Box::new(target),
                        }))
                    }
                    _ => Ok(Stmt::Expression(Expr::Identifier(id))),
                }
            }
            Some(t) => Err(ParseError::UnexpectedToken(format!("{:?}", t))),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn parse_confidence_declaration(&mut self) -> Result<Stmt, ParseError> {
        self.tokens.next(); // consume 'conf'
        let name = self.parse_identifier()?;
        
        match self.tokens.next() {
            Some(Token::Assign) => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        let value = self.parse_expression()?;
        Ok(Stmt::Declaration { name, value })
    }

    fn parse_uncertain_if(&mut self) -> Result<Stmt, ParseError> {
        self.tokens.next(); // consume 'uncertain'
        match self.tokens.next() {
            Some(Token::If) => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        match self.tokens.next() {
            Some(Token::LParen) => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        let condition = self.parse_expression()?;

        match self.tokens.next() {
            Some(Token::RParen) => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        let high_confidence = self.parse_block()?;
        
        let medium_confidence = if let Some(Token::Medium) = self.tokens.peek() {
            self.tokens.next(); // consume 'medium'
            Some(self.parse_block()?)
        } else {
            None
        };

        let low_confidence = if let Some(Token::Low) = self.tokens.peek() {
            self.tokens.next(); // consume 'low'
            Some(self.parse_block()?)
        } else {
            None
        };

        Ok(Stmt::Expression(Expr::UncertainIf {
            condition: Box::new(condition),
            high_confidence,
            medium_confidence,
            low_confidence,
        }))
    }

    fn parse_context_block(&mut self) -> Result<Stmt, ParseError> {
        self.tokens.next(); // consume 'in'
        match self.tokens.next() {
            Some(Token::Context) => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        let context_name = self.parse_identifier()?;
        let body = self.parse_block()?;

        Ok(Stmt::Expression(Expr::ContextBlock {
            context_name,
            body,
        }))
    }

    fn parse_verification(&mut self) -> Result<Stmt, ParseError> {
        self.tokens.next(); // consume 'verify'
        match self.tokens.next() {
            Some(Token::Identifier(s)) if s == "against" => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        match self.tokens.next() {
            Some(Token::Identifier(s)) if s == "sources" => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        let sources = self.parse_string_array()?;
        let threshold = match self.tokens.next() {
            Some(Token::Float(s)) => s.parse::<f64>().map_err(|_| ParseError::InvalidNumber(s))?,
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        };

        let body = self.parse_block()?;

        Ok(Stmt::Expression(Expr::Verify {
            sources,
            threshold,
            body,
        }))
    }

    fn parse_string_array(&mut self) -> Result<Vec<String>, ParseError> {
        match self.tokens.next() {
            Some(Token::LBracket) => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        let mut strings = Vec::new();
        loop {
            match self.tokens.next() {
                Some(Token::String(s)) => strings.push(s),
                Some(Token::RBracket) => break,
                t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
            }
        }

        Ok(strings)
    }

    fn parse_block(&mut self) -> Result<Vec<Stmt>, ParseError> {
        match self.tokens.next() {
            Some(Token::LBrace) => (),
            t => return Err(ParseError::UnexpectedToken(format!("{:?}", t))),
        }

        let mut statements = Vec::new();
        while let Some(token) = self.tokens.peek() {
            if *token == Token::RBrace {
                self.tokens.next(); // consume '}'
                return Ok(statements);
            }
            statements.push(self.parse_statement()?);
        }

        Err(ParseError::UnexpectedEOF)
    }

    fn parse_expression(&mut self) -> Result<Expr, ParseError> {
        let left = self.parse_primary()?;

        match self.tokens.peek() {
            Some(Token::ConfidenceFlow) => {
                self.tokens.next(); // consume ~>
                let right = self.parse_expression()?;
                Ok(Expr::ConfidenceFlow {
                    source: Box::new(left),
                    target: Box::new(right),
                })
            }
            Some(Token::ReverseConfidenceFlow) => {
                self.tokens.next(); // consume <~
                let right = self.parse_expression()?;
                Ok(Expr::ReverseConfidenceFlow {
                    target: Box::new(left),
                    source: Box::new(right),
                })
            }
            Some(Token::ConfidenceAnd) => {
                self.tokens.next();
                let right = self.parse_expression()?;
                Ok(Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::ConfidenceAnd,
                    right: Box::new(right),
                })
            }
            Some(Token::ConfidenceOr) => {
                self.tokens.next();
                let right = self.parse_expression()?;
                Ok(Expr::BinaryOp {
                    left: Box::new(left),
                    op: BinaryOperator::ConfidenceOr,
                    right: Box::new(right),
                })
            }
            _ => Ok(left),
        }
    }

    fn parse_primary(&mut self) -> Result<Expr, ParseError> {
        match self.tokens.next() {
            Some(Token::Float(s)) => {
                let value = s.parse::<f64>()
                    .map_err(|_| ParseError::InvalidNumber(s.clone()))?;
                Ok(Expr::Float(value))
            }
            Some(Token::Integer(s)) => {
                let value = s.parse::<i64>()
                    .map_err(|_| ParseError::InvalidNumber(s.clone()))?;
                Ok(Expr::Integer(value))
            }
            Some(Token::String(s)) => Ok(Expr::String(s)),
            Some(Token::Identifier(s)) => Ok(Expr::Identifier(s)),
            Some(t) => Err(ParseError::UnexpectedToken(format!("{:?}", t))),
            None => Err(ParseError::UnexpectedEOF),
        }
    }

    fn parse_identifier(&mut self) -> Result<String, ParseError> {
        match self.tokens.next() {
            Some(Token::Identifier(s)) => Ok(s),
            Some(t) => Err(ParseError::UnexpectedToken(format!("{:?}", t))),
            None => Err(ParseError::UnexpectedEOF),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::Lexer;

    #[test]
    fn test_parse_confidence_declaration() {
        let input = "conf x = 0.8";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement().unwrap();
        assert!(matches!(
            stmt,
            Stmt::Declaration {
                name: _,
                value: Expr::Float(_)
            }
        ));
    }

    #[test]
    fn test_parse_confidence_flow() {
        let input = "x ~> 0.7";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement().unwrap();
        assert!(matches!(
            stmt,
            Stmt::Expression(Expr::ConfidenceFlow { .. })
        ));
    }

    #[test]
    fn test_parse_uncertain_if() {
        let input = "uncertain if (x ~> 0.7) { conf y = 0.8 }";
        let lexer = Lexer::new(input);
        let mut parser = Parser::new(lexer);
        
        let stmt = parser.parse_statement().unwrap();
        assert!(matches!(
            stmt,
            Stmt::Expression(Expr::UncertainIf { .. })
        ));
    }
} 