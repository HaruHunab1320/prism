use std::sync::Arc;
use parking_lot::RwLock;
use crate::ast::{Expr, Stmt};
use crate::environment::Environment;
use crate::error::{PrismError, Result};
use crate::value::{Value, ValueKind};
use crate::token::TokenKind;
use std::future::Future;
use std::pin::Pin;

pub struct Interpreter {
    environment: Arc<RwLock<Environment>>,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Arc::new(RwLock::new(Environment::new())),
        }
    }

    pub async fn evaluate(&mut self, source: String) -> Result<Value> {
        let statements = crate::parser::parse(&source)?;
        let mut result = Value::new(ValueKind::Nil);
        for stmt in statements {
            result = self.execute_statement(&stmt).await?;
        }
        Ok(result)
    }

    fn execute_statement<'a>(&'a mut self, stmt: &'a Stmt) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            match stmt {
                Stmt::Expression(expr) => {
                    println!("Executing expression: {:?}", expr);
                    let result = self.evaluate_expression(expr).await?;
                    println!("Expression result: {:?}", result);
                    Ok(result)
                },
                Stmt::Let(name, initializer) => {
                    println!("Declaring variable: {} with initializer: {:?}", name, initializer);
                    let value = if let Some(init) = initializer {
                        let val = self.evaluate_expression(init).await?;
                        println!("Initialized {} with value: {:?}", name, val);
                        val
                    } else {
                        Value::new(ValueKind::Nil)
                    };
                    self.environment.write().define(name.clone(), value.clone())?;
                    Ok(value)
                },
                Stmt::Block(statements) => {
                    let mut result = Value::new(ValueKind::Nil);
                    for stmt in statements {
                        result = self.execute_statement(stmt).await?;
                    }
                    Ok(result)
                },
                Stmt::Function { name, params, body: _, is_async: _, confidence } => {
                    let closure = Arc::clone(&self.environment);
                    let params = params.clone();
                    let mut function = Value::new(ValueKind::Function {
                        name: name.clone(),
                        params: params.clone(),
                        body: Arc::new(move |args| {
                            let mut env = Environment::with_enclosing(Arc::clone(&closure));
                            for (param, arg) in params.iter().zip(args) {
                                env.define(param.clone(), arg)?;
                            }
                            Ok(Value::new(ValueKind::Nil)) // Placeholder
                        }),
                    });
                    if let Some(conf) = confidence {
                        function.set_confidence(*conf);
                    }
                    self.environment.write().define(name.clone(), function.clone())?;
                    Ok(function)
                },
                _ => Ok(Value::new(ValueKind::Nil)), // Handle other statement types
            }
        })
    }

    fn evaluate_expression<'a>(&'a self, expr: &'a Expr) -> Pin<Box<dyn Future<Output = Result<Value>> + Send + 'a>> {
        Box::pin(async move {
            match expr {
                Expr::Literal(value) => {
                    println!("Evaluating literal: {:?}", value);
                    Ok(value.clone())
                },
                Expr::Variable(name) => {
                    println!("Looking up variable: {}", name);
                    let val = self.environment.read().get(name)?;
                    println!("Found value: {:?}", val);
                    Ok(val)
                },
                Expr::Grouping(expr) => {
                    println!("Evaluating grouped expression: {:?}", expr);
                    self.evaluate_expression(expr).await
                },
                Expr::Binary { left, operator, right } => {
                    println!("Evaluating binary expression: {:?} {:?} {:?}", left, operator, right);
                    let left = self.evaluate_expression(left).await?;
                    let right = self.evaluate_expression(right).await?;
                    println!("Binary operands: {:?} {:?}", left, right);
                    
                    match (&left.kind, &right.kind) {
                        (ValueKind::Number(l), ValueKind::Number(r)) => {
                            let result = match operator.kind {
                                TokenKind::Plus => l + r,
                                TokenKind::Minus => l - r,
                                TokenKind::Star => l * r,
                                TokenKind::Slash => l / r,
                                _ => return Err(PrismError::RuntimeError("Invalid operator for numbers".to_string())),
                            };
                            println!("Binary result: {:?}", result);
                            Ok(Value::new(ValueKind::Number(result)))
                        },
                        _ => Err(PrismError::RuntimeError(format!("Operands must be numbers, got {:?} and {:?}", left.kind, right.kind))),
                    }
                },
                Expr::Assign { name, value } => {
                    let value = self.evaluate_expression(value).await?;
                    self.environment.write().assign(name, value.clone())?;
                    Ok(value)
                },
                Expr::Call { callee, arguments } => {
                    let callee = self.evaluate_expression(callee).await?;
                    let mut args = Vec::new();
                    for arg in arguments {
                        args.push(self.evaluate_expression(arg).await?);
                    }
                    match callee.kind {
                        ValueKind::Function { ref body, .. } => body(args),
                        ValueKind::NativeFunction { ref handler, .. } => handler(args),
                        _ => Err(PrismError::RuntimeError("Not a callable value".to_string())),
                    }
                }
                _ => Ok(Value::new(ValueKind::Nil)), // Handle other expression types
            }
        })
    }
}
