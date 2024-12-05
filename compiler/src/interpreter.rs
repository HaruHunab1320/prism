use std::error::Error;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

use crate::ast::{Expr, Stmt};
use crate::context::ContextManager;
use crate::environment::Environment;
use crate::lexer::Lexer;
use crate::llm::LLMClient;
use crate::parser::Parser;
use crate::types::Value;

pub struct Interpreter {
    env: Environment,
    llm_client: LLMClient,
    context_manager: ContextManager,
}

impl Interpreter {
    pub fn new(api_key: String) -> Self {
        Self {
            env: Environment::new(),
            llm_client: LLMClient::new(api_key),
            context_manager: ContextManager::new(0.8),
        }
    }

    pub async fn eval(&mut self, source: String) -> Result<Value, Box<dyn Error>> {
        let mut lexer = Lexer::new(&source);
        let (tokens, starts, ends) = lexer.lex()?;
        let mut parser = Parser::new(source, tokens, starts, ends);
        let statements = parser.parse()?;
        
        let mut last_value = Value::Void;
        for stmt in statements {
            last_value = self.eval_stmt_boxed(stmt).await?;
        }
        Ok(last_value)
    }

    pub fn register_native_function<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&mut Interpreter, Vec<Value>) -> Result<Value, Box<dyn Error>> + Send + Sync + 'static,
    {
        self.env.define(name, Value::NativeFunction(Arc::new(f)));
    }

    pub async fn eval_stmt(&mut self, stmt: Stmt) -> Result<Value, Box<dyn Error>> {
        self.eval_stmt_boxed(stmt).await
    }

    pub async fn eval_expr(&mut self, expr: Expr) -> Result<Value, Box<dyn Error>> {
        self.eval_expr_boxed(expr).await
    }

    fn eval_stmt_boxed(&mut self, stmt: Stmt) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error>>> + '_>> {
        Box::pin(async move {
            match stmt {
                Stmt::Expression(expr) => self.eval_expr_boxed(expr).await,
                Stmt::Let(name, initializer) => {
                    let value = match initializer {
                        Some(expr) => self.eval_expr_boxed(expr).await?,
                        None => Value::Void,
                    };
                    self.env.define(&name, value.clone());
                    Ok(value)
                },
                Stmt::Block(statements) => {
                    let mut result = Value::Void;
                    for stmt in statements {
                        result = self.eval_stmt_boxed(stmt).await?;
                    }
                    Ok(result)
                },
                Stmt::Context(name, body) => {
                    let old_context = self.env.get_current_context().cloned();
                    self.env.set_context(Some(name));
                    let result = self.eval_stmt_boxed(*body).await;
                    self.env.set_context(old_context);
                    result
                },
                Stmt::Verify(_sources, body) => {
                    let old_threshold = self.context_manager.get_threshold();
                    self.context_manager.set_threshold(0.9);
                    let result = self.eval_stmt_boxed(*body).await;
                    self.context_manager.set_threshold(old_threshold);
                    result
                },
                Stmt::UncertainIf(condition, then_branch, medium_branch, else_branch) => {
                    let cond_value = self.eval_expr_boxed(condition).await?;
                    let confidence = match &cond_value {
                        Value::Float(n) => *n,
                        _ => return Err("Condition must evaluate to a confidence value".into()),
                    };

                    if confidence > 0.8 {
                        self.eval_stmt_boxed(*then_branch).await
                    } else if confidence > 0.5 && medium_branch.is_some() {
                        self.eval_stmt_boxed(*medium_branch.unwrap()).await
                    } else if else_branch.is_some() {
                        self.eval_stmt_boxed(*else_branch.unwrap()).await
                    } else {
                        Ok(Value::Void)
                    }
                },
                Stmt::TryConfidence { body, below_threshold, uncertain, threshold } => {
                    let result = self.eval_stmt_boxed(*body).await;
                    match result {
                        Ok(value) => {
                            let confidence = value.get_confidence().unwrap_or(1.0);
                            if confidence < threshold {
                                self.eval_stmt_boxed(*below_threshold).await
                            } else {
                                Ok(value)
                            }
                        }
                        Err(_) => self.eval_stmt_boxed(*uncertain).await,
                    }
                },
                _ => Ok(Value::Void),
            }
        })
    }

    fn eval_expr_boxed(&mut self, expr: Expr) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error>>> + '_>> {
        Box::pin(async move {
            match expr {
                Expr::Literal(value) => Ok(value),
                Expr::Variable(name) => {
                    self.env.get(&name).cloned().ok_or_else(|| format!("Undefined variable '{}'", name).into())
                },
                Expr::Binary(left, operator, right) => {
                    let left = self.eval_expr_boxed(*left).await?;
                    let right = self.eval_expr_boxed(*right).await?;
                    let left_clone = left.clone();
                    let right_clone = right.clone();
                    match (left, operator.as_str(), right) {
                        (Value::Float(l), "+", Value::Float(r)) => Ok(Value::Float(l + r)),
                        (Value::Float(l), "-", Value::Float(r)) => Ok(Value::Float(l - r)),
                        (Value::Float(l), "*", Value::Float(r)) => Ok(Value::Float(l * r)),
                        (Value::Float(l), "/", Value::Float(r)) => {
                            if r == 0.0 {
                                Err("Division by zero".into())
                            } else {
                                Ok(Value::Float(l / r))
                            }
                        },
                        (Value::Float(l), ">", Value::Float(r)) => Ok(Value::Boolean(l > r)),
                        (Value::Float(l), ">=", Value::Float(r)) => Ok(Value::Boolean(l >= r)),
                        (Value::Float(l), "<", Value::Float(r)) => Ok(Value::Boolean(l < r)),
                        (Value::Float(l), "<=", Value::Float(r)) => Ok(Value::Boolean(l <= r)),
                        (Value::Float(l), "==", Value::Float(r)) => Ok(Value::Boolean(l == r)),
                        (Value::Float(l), "!=", Value::Float(r)) => Ok(Value::Boolean(l != r)),
                        _ => Err(format!("Invalid binary operation: {:?} {} {:?}", left_clone, operator, right_clone).into()),
                    }
                },
                Expr::Unary(operator, expr) => {
                    let value = self.eval_expr_boxed(*expr).await?;
                    let value_clone = value.clone();
                    match (operator.as_str(), value) {
                        ("-", Value::Float(n)) => Ok(Value::Float(-n)),
                        ("!", Value::Boolean(b)) => Ok(Value::Boolean(!b)),
                        _ => Err(format!("Invalid unary operation: {} {:?}", operator, value_clone).into()),
                    }
                },
                Expr::Grouping(expr) => self.eval_expr_boxed(*expr).await,
                Expr::Call(callee, arguments) => {
                    let callee = self.eval_expr_boxed(*callee).await?;
                    let mut args = Vec::new();
                    for arg in arguments {
                        args.push(self.eval_expr_boxed(arg).await?);
                    }
                    match callee {
                        Value::NativeFunction(f) => f(self, args),
                        Value::AsyncFn(f) => f(args).await,
                        _ => Err("Can only call functions".into()),
                    }
                },
                Expr::Get(object, name) => {
                    let object = self.eval_expr_boxed(*object).await?;
                    match object {
                        Value::Tensor(values, shape) => {
                            match name.as_str() {
                                "cosine_similarity" => Ok(Value::NativeFunction(Arc::new(move |_: &mut Interpreter, args: Vec<Value>| {
                                    if args.len() != 1 {
                                        return Err("cosine_similarity takes exactly one argument".into());
                                    }
                                    match &args[0] {
                                        Value::Tensor(other_values, other_shape) if shape == *other_shape => {
                                            let dot_product: f64 = values.iter().zip(other_values.iter()).map(|(a, b)| a * b).sum();
                                            let norm1: f64 = values.iter().map(|x| x * x).sum::<f64>().sqrt();
                                            let norm2: f64 = other_values.iter().map(|x| x * x).sum::<f64>().sqrt();
                                            
                                            if norm1 == 0.0 || norm2 == 0.0 {
                                                Ok(Value::Float(0.0))
                                            } else {
                                                Ok(Value::Float(dot_product / (norm1 * norm2)))
                                            }
                                        },
                                        _ => Err("Cosine similarity requires tensors of same shape".into()),
                                    }
                                }))),
                                _ => Err(format!("No such method '{}' on tensor", name).into()),
                            }
                        },
                        _ => Err(format!("Cannot get property '{}' of {:?}", name, object).into()),
                    }
                },
                Expr::ConfidenceFlow(expr, confidence) => {
                    let value = self.eval_expr_boxed(*expr).await?;
                    let conf = self.eval_expr_boxed(*confidence).await?;
                    match conf {
                        Value::Float(n) if n >= 0.0 && n <= 1.0 => value.with_confidence(n),
                        _ => Err("Confidence must be a float between 0 and 1".into()),
                    }
                },
                Expr::SemanticMatch(left, right) => {
                    let left = self.eval_expr_boxed(*left).await?;
                    let right = self.eval_expr_boxed(*right).await?;
                    match (left, right) {
                        (Value::String(pattern), Value::String(text)) => {
                            let confidence = self.llm_client.semantic_match(&text, &pattern).await?;
                            Ok(Value::Float(confidence))
                        },
                        _ => Err("Semantic match requires string operands".into()),
                    }
                },
                Expr::Tensor { values, shape } => {
                    let mut tensor_values = Vec::new();
                    for value in values.iter() {
                        let val = self.eval_expr_boxed(value.clone()).await?;
                        match val {
                            Value::Float(n) => tensor_values.push(n),
                            _ => return Err("Tensor values must be floats".into()),
                        }
                    }

                    let tensor = match shape {
                        Some(dims) => {
                            let total_size: usize = dims.iter().product();
                            if total_size != tensor_values.len() {
                                return Err("Tensor shape does not match number of values".into());
                            }
                            Value::Tensor(tensor_values, dims)
                        },
                        None => {
                            let len = tensor_values.len();
                            Value::Tensor(tensor_values, vec![len])
                        }
                    };

                    Ok(tensor)
                },
                _ => Ok(Value::Void),
            }
        })
    }
}