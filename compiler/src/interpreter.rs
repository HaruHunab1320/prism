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
                    self.evaluate_expression(expr).await
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
                Stmt::If { condition, then_branch, else_branch } => {
                    println!("Executing if statement with condition: {:?}", condition);
                    let cond_value = self.evaluate_expression(condition).await?;
                    
                    match cond_value.kind {
                        ValueKind::Boolean(true) => {
                            println!("Condition is true, executing then branch");
                            self.execute_statement(then_branch).await
                        },
                        ValueKind::Boolean(false) => {
                            if let Some(else_stmt) = else_branch {
                                println!("Condition is false, executing else branch");
                                self.execute_statement(else_stmt).await
                            } else {
                                println!("Condition is false, no else branch");
                                Ok(Value::new(ValueKind::Nil))
                            }
                        },
                        _ => Err(PrismError::RuntimeError(format!("Condition must be a boolean, got {:?}", cond_value.kind))),
                    }
                },
                Stmt::Block(statements) => {
                    println!("Executing block with {} statements", statements.len());
                    // Create a new environment for this block
                    let previous = Arc::clone(&self.environment);
                    self.environment = Arc::new(RwLock::new(Environment::with_enclosing(previous)));
                    
                    let mut result = Value::new(ValueKind::Nil);
                    for stmt in statements {
                        result = self.execute_statement(stmt).await?;
                    }
                    
                    // Restore the previous environment
                    let enclosing = {
                        let env = self.environment.read();
                        env.get_enclosing()
                    };
                    
                    if let Some(parent_env) = enclosing {
                        self.environment = parent_env;
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
                        // Numeric operations
                        (ValueKind::Number(l), ValueKind::Number(r)) => {
                            let result = match operator.kind {
                                TokenKind::Plus => Value::new(ValueKind::Number(l + r)),
                                TokenKind::Minus => Value::new(ValueKind::Number(l - r)),
                                TokenKind::Star => Value::new(ValueKind::Number(l * r)),
                                TokenKind::Slash => Value::new(ValueKind::Number(l / r)),
                                // Comparison operators
                                TokenKind::Greater => Value::new(ValueKind::Boolean(l > r)),
                                TokenKind::GreaterEqual => Value::new(ValueKind::Boolean(l >= r)),
                                TokenKind::Less => Value::new(ValueKind::Boolean(l < r)),
                                TokenKind::LessEqual => Value::new(ValueKind::Boolean(l <= r)),
                                TokenKind::EqualEqual => Value::new(ValueKind::Boolean(l == r)),
                                TokenKind::BangEqual => Value::new(ValueKind::Boolean(l != r)),
                                _ => return Err(PrismError::RuntimeError("Invalid operator for numbers".to_string())),
                            };
                            println!("Binary result: {:?}", result);
                            Ok(result)
                        },
                        // Boolean operations
                        (ValueKind::Boolean(l), ValueKind::Boolean(r)) => {
                            let result = match operator.kind {
                                TokenKind::And => Value::new(ValueKind::Boolean(*l && *r)),
                                TokenKind::Or => Value::new(ValueKind::Boolean(*l || *r)),
                                TokenKind::EqualEqual => Value::new(ValueKind::Boolean(l == r)),
                                TokenKind::BangEqual => Value::new(ValueKind::Boolean(l != r)),
                                _ => return Err(PrismError::RuntimeError("Invalid operator for booleans".to_string())),
                            };
                            println!("Binary result: {:?}", result);
                            Ok(result)
                        },
                        // String operations
                        (ValueKind::String(l), ValueKind::String(r)) => {
                            let result = match operator.kind {
                                TokenKind::Plus => Value::new(ValueKind::String(format!("{}{}", l, r))),
                                TokenKind::EqualEqual => Value::new(ValueKind::Boolean(l == r)),
                                TokenKind::BangEqual => Value::new(ValueKind::Boolean(l != r)),
                                _ => return Err(PrismError::RuntimeError("Invalid operator for strings".to_string())),
                            };
                            println!("Binary result: {:?}", result);
                            Ok(result)
                        },
                        // Equality for any type
                        _ => match operator.kind {
                            TokenKind::EqualEqual => Ok(Value::new(ValueKind::Boolean(left.kind == right.kind))),
                            TokenKind::BangEqual => Ok(Value::new(ValueKind::Boolean(left.kind != right.kind))),
                            _ => Err(PrismError::RuntimeError(format!(
                                "Invalid operation between {:?} and {:?}",
                                left.kind, right.kind
                            ))),
                        },
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
