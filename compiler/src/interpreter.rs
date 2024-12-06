use crate::ast::{Expr, Stmt};
use crate::error::Error;
use crate::lexer::TokenType;
use crate::value::Value;
use std::error::Error as StdError;
use std::future::Future;
use std::pin::Pin;

pub type AsyncResult<T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn StdError + Send + Sync>>> + Send>>;

pub struct Interpreter {
    api_key: String,
}

impl Interpreter {
    pub fn new(api_key: String) -> Self {
        Self { api_key }
    }

    pub async fn eval(&self, source: String) -> Result<Value, Box<dyn StdError + Send + Sync>> {
        let mut lexer = crate::lexer::Lexer::new(&source);
        let tokens = lexer.lex()?;
        let mut parser = crate::parser::Parser::new(tokens);
        let statements = parser.parse()?;
        let mut last_value = Value::Null;

        for stmt in statements {
            last_value = self.eval_statement(&stmt).await?;
        }

        Ok(last_value)
    }

    async fn eval_statement(&self, stmt: &Stmt) -> Result<Value, Box<dyn StdError + Send + Sync>> {
        let future = Box::pin(async move {
            match stmt {
                Stmt::Expression(expr) => self.eval_expression(expr).await,
                Stmt::Let(name, initializer) => {
                    let value = match initializer {
                        Some(expr) => self.eval_expression(expr).await?,
                        None => Value::Null,
                    };
                    Ok(value)
                },
                Stmt::Block(statements) => {
                    let mut result = Value::Null;
                    for statement in statements {
                        result = self.eval_statement(statement).await?;
                    }
                    Ok(result)
                },
                Stmt::If { condition, then_branch, else_branch } => {
                    let condition_value = self.eval_expression(condition).await?;
                    if is_truthy(&condition_value) {
                        self.eval_statement(then_branch).await
                    } else if let Some(else_branch) = else_branch {
                        self.eval_statement(else_branch).await
                    } else {
                        Ok(Value::Null)
                    }
                },
                Stmt::While { condition, body } => {
                    let mut result = Value::Null;
                    while is_truthy(&self.eval_expression(condition).await?) {
                        result = self.eval_statement(body).await?;
                    }
                    Ok(result)
                },
                Stmt::Function { name: _, params: _, body: _, is_async: _ } => {
                    Ok(Value::Null) // Function declarations don't produce a value
                },
                Stmt::Return(value) => {
                    match value {
                        Some(expr) => self.eval_expression(expr).await,
                        None => Ok(Value::Null),
                    }
                },
            }
        });
        future.await
    }

    pub async fn eval_expression(&self, expr: &Expr) -> Result<Value, Box<dyn StdError + Send + Sync>> {
        let future = Box::pin(async move {
            match expr {
                Expr::Literal(value) => Ok(value.clone()),
                Expr::Variable(name) => Ok(Value::String(name.clone())),
                Expr::Binary { left, operator, right } => {
                    let left_val = self.eval_expression(left).await?;
                    let right_val = self.eval_expression(right).await?;
                    
                    match &operator.token_type {
                        TokenType::Plus => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
                            _ => Err(Box::new(Error::new("Invalid operands for '+' operator")) as Box<dyn StdError + Send + Sync>)
                        },
                        TokenType::Minus => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                            _ => Err(Box::new(Error::new("Invalid operands for '-' operator")) as Box<dyn StdError + Send + Sync>)
                        },
                        TokenType::Star => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                            _ => Err(Box::new(Error::new("Invalid operands for '*' operator")) as Box<dyn StdError + Send + Sync>)
                        },
                        TokenType::Slash => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                            _ => Err(Box::new(Error::new("Invalid operands for '/' operator")) as Box<dyn StdError + Send + Sync>)
                        },
                        TokenType::Greater => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
                            _ => Err(Box::new(Error::new("Invalid operands for '>' operator")) as Box<dyn StdError + Send + Sync>)
                        },
                        TokenType::GreaterEqual => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
                            _ => Err(Box::new(Error::new("Invalid operands for '>=' operator")) as Box<dyn StdError + Send + Sync>)
                        },
                        TokenType::Less => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
                            _ => Err(Box::new(Error::new("Invalid operands for '<' operator")) as Box<dyn StdError + Send + Sync>)
                        },
                        TokenType::LessEqual => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
                            _ => Err(Box::new(Error::new("Invalid operands for '<=' operator")) as Box<dyn StdError + Send + Sync>)
                        },
                        _ => Err(Box::new(Error::new("Invalid binary operator")) as Box<dyn StdError + Send + Sync>)
                    }
                },
                Expr::Logical { left, operator, right } => {
                    let left_val = self.eval_expression(left).await?;
                    
                    match &operator.token_type {
                        TokenType::And => {
                            if !is_truthy(&left_val) {
                                Ok(left_val)
                            } else {
                                self.eval_expression(right).await
                            }
                        },
                        TokenType::Or => {
                            if is_truthy(&left_val) {
                                Ok(left_val)
                            } else {
                                self.eval_expression(right).await
                            }
                        },
                        _ => Err(Box::new(Error::new("Invalid logical operator")) as Box<dyn StdError + Send + Sync>)
                    }
                },
                _ => Err(Box::new(Error::new("Invalid expression")) as Box<dyn StdError + Send + Sync>)
            }
        });
        future.await
    }
}

fn is_truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null => false,
        _ => true,
    }
}
