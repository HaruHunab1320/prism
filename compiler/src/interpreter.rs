use crate::ast::{BinaryOp, Expr, UnaryOp, Value};
use crate::environment::Environment;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub struct Interpreter {
    pub environment: Environment,
    api_key: String,
}

impl Interpreter {
    pub fn new(api_key: String) -> Self {
        Self {
            environment: Environment::new(),
            api_key,
        }
    }

    pub async fn eval(&mut self, source: String) -> Result<Value, Box<dyn Error + Send + Sync>> {
        // ... existing code ...
        Ok(Value::Null)
    }

    pub fn eval_expr<'a>(
        &'a mut self,
        expr: Expr,
    ) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error + Send + Sync>>> + Send + 'a>>
    {
        Box::pin(async move {
            match expr {
                Expr::Literal(value) => Ok(value),
                Expr::Variable(name) => {
                    if let Some(value) = self.environment.get(&name) {
                        Ok(value)
                    } else {
                        Err(format!("Undefined variable '{}'.", name).into())
                    }
                }
                Expr::Unary { operator, expr } => {
                    let right = self.eval_expr(*expr).await?;
                    match operator {
                        UnaryOp::Not => {
                            if let Value::Bool(b) = right {
                                Ok(Value::Bool(!b))
                            } else {
                                Err("Operand must be a boolean.".into())
                            }
                        }
                        UnaryOp::Minus => {
                            if let Value::Number(n) = right {
                                Ok(Value::Number(-n))
                            } else {
                                Err("Operand must be a number.".into())
                            }
                        }
                    }
                }
                Expr::Binary {
                    left,
                    operator,
                    right,
                } => {
                    let left = self.eval_expr(*left).await?;
                    let right = self.eval_expr(*right).await?;
                    match operator {
                        BinaryOp::Add => match (left, right) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                            (Value::String(a), Value::String(b)) => {
                                Ok(Value::String(format!("{}{}", a, b)))
                            }
                            _ => Err("Operands must be two numbers or two strings.".into()),
                        },
                        BinaryOp::Subtract => match (left, right) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                            _ => Err("Operands must be numbers.".into()),
                        },
                        BinaryOp::Multiply => match (left, right) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                            _ => Err("Operands must be numbers.".into()),
                        },
                        BinaryOp::Divide => match (left, right) {
                            (Value::Number(a), Value::Number(b)) => {
                                if b == 0.0 {
                                    Err("Division by zero.".into())
                                } else {
                                    Ok(Value::Number(a / b))
                                }
                            }
                            _ => Err("Operands must be numbers.".into()),
                        },
                        BinaryOp::Equal => Ok(Value::Bool(left == right)),
                        BinaryOp::NotEqual => Ok(Value::Bool(left != right)),
                        BinaryOp::Greater => match (left, right) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
                            _ => Err("Operands must be numbers.".into()),
                        },
                        BinaryOp::GreaterEqual => match (left, right) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
                            _ => Err("Operands must be numbers.".into()),
                        },
                        BinaryOp::Less => match (left, right) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
                            _ => Err("Operands must be numbers.".into()),
                        },
                        BinaryOp::LessEqual => match (left, right) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
                            _ => Err("Operands must be numbers.".into()),
                        },
                        BinaryOp::And => match (left, right) {
                            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                            _ => Err("Operands must be booleans.".into()),
                        },
                        BinaryOp::Or => match (left, right) {
                            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                            _ => Err("Operands must be booleans.".into()),
                        },
                    }
                }
                Expr::Call { callee, arguments } => {
                    let callee = self.eval_expr(*callee).await?;
                    let mut evaluated_args = Vec::new();
                    for arg in arguments {
                        evaluated_args.push(self.eval_expr(arg).await?);
                    }
                    match callee {
                        Value::AsyncFn(func) => func(self, evaluated_args).await,
                        _ => Err("Can only call functions.".into()),
                    }
                }
                Expr::Match { value, arms } => {
                    let value = self.eval_expr(*value).await?;
                    for (pattern, expr) in arms {
                        let pattern_value = self.eval_expr(pattern).await?;
                        if value == pattern_value {
                            return self.eval_expr(*expr).await;
                        }
                    }
                    Ok(Value::Null)
                }
                Expr::TryConfidence {
                    body,
                    threshold,
                    fallback,
                    error_handler,
                } => match self.eval_expr(*body).await {
                    Ok(value) => {
                        if let Some(confidence) = value.confidence() {
                            if confidence >= threshold {
                                Ok(value)
                            } else {
                                self.eval_expr(*fallback).await
                            }
                        } else {
                            self.eval_expr(*fallback).await
                        }
                    }
                    Err(e) => {
                        if let Some(handler) = error_handler {
                            self.eval_expr(*handler).await
                        } else {
                            Err(e)
                        }
                    }
                },
                Expr::Verify { value, pattern } => {
                    let value = self.eval_expr(*value).await?;
                    let pattern = self.eval_expr(*pattern).await?;
                    // TODO: Implement semantic pattern matching
                    Ok(Value::Bool(value == pattern))
                }
            }
        })
    }

    pub fn register_native_function<F>(&mut self, name: &str, function: F)
    where
        F: Fn(
                &mut Interpreter,
                Vec<Value>,
            ) -> Pin<
                Box<dyn Future<Output = Result<Value, Box<dyn Error + Send + Sync>>> + Send + Sync>,
            > + Send
            + Sync
            + 'static,
    {
        self.environment
            .define(name.to_string(), Value::AsyncFn(Arc::new(function)));
    }
}
