use std::error::Error;
use std::collections::HashMap;
use std::sync::Arc;
use std::pin::Pin;
use std::future::Future;

use crate::ast::{Expr, Stmt};
use crate::value::Value;
use crate::llm::LLMClient;
use crate::stdlib;

#[derive(Debug, Clone)]
pub struct Context {
    pub name: String,
    pub confidence: f64,
}

#[derive(Debug)]
pub struct ContextManager {
    contexts: Vec<Context>,
}

impl ContextManager {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }

    pub fn push_context(&mut self, name: String, confidence: f64) {
        self.contexts.push(Context { name, confidence });
    }

    pub fn pop_context(&mut self) -> Option<Context> {
        self.contexts.pop()
    }

    pub fn current_context(&self) -> Option<&Context> {
        self.contexts.last()
    }
}

#[derive(Debug, Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    parent: Option<Box<Environment>>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            parent: None,
        }
    }

    pub fn with_parent(parent: Environment) -> Self {
        Self {
            values: HashMap::new(),
            parent: Some(Box::new(parent)),
        }
    }

    pub fn define(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        match self.values.get(name) {
            Some(value) => Some(value.clone()),
            None => self.parent.as_ref().and_then(|p| p.get(name)),
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), Box<dyn Error>> {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            Ok(())
        } else if let Some(parent) = &mut self.parent {
            parent.assign(name, value)
        } else {
            Err(format!("Undefined variable '{}'.", name).into())
        }
    }
}

pub struct Interpreter {
    environment: Environment,
    context_manager: ContextManager,
    llm_client: LLMClient,
}

impl Interpreter {
    pub fn new(api_key: String) -> Self {
        let mut interpreter = Self {
            environment: Environment::new(),
            context_manager: ContextManager::new(),
            llm_client: LLMClient::new(api_key),
        };

        stdlib::register_core_functions(&mut interpreter);
        stdlib::register_utils_functions(&mut interpreter);

        interpreter
    }

    pub fn register_native_function<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&mut Interpreter, Vec<Value>) -> Result<Value, Box<dyn Error>> + Send + Sync + 'static,
    {
        self.environment.define(name.to_string(), Value::NativeFunction(Arc::new(f)));
    }

    pub async fn eval(&mut self, source: String) -> Result<Value, Box<dyn Error>> {
        let mut lexer = crate::lexer::Lexer::new(&source);
        let (tokens, starts, ends) = lexer.lex()?;
        let mut parser = crate::parser::Parser::new(source, tokens, starts, ends);
        let statements = parser.parse()?;

        let mut last_value = Value::None;
        for stmt in statements {
            last_value = self.eval_stmt(stmt).await?;
        }
        Ok(last_value)
    }

    pub fn eval_stmt(&mut self, stmt: Stmt) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error>>> + '_>> {
        Box::pin(async move {
            match stmt {
                Stmt::Expression(expr) => self.eval_expr(expr).await,
                Stmt::Let(name, initializer) => {
                    let value = self.eval_expr(initializer).await?;
                    self.environment.define(name, value.clone());
                    Ok(value)
                }
                Stmt::Block(statements) => {
                    let mut last_value = Value::None;
                    let mut scope = Environment::with_parent(self.environment.clone());
                    std::mem::swap(&mut self.environment, &mut scope);

                    for stmt in statements {
                        last_value = self.eval_stmt(stmt).await?;
                    }

                    std::mem::swap(&mut self.environment, &mut scope);
                    Ok(last_value)
                }
                Stmt::Context(name, body) => {
                    self.context_manager.push_context(name, 1.0);
                    let result = self.eval_stmt(*body).await?;
                    self.context_manager.pop_context();
                    Ok(result)
                }
                Stmt::ContextTransition { from_context, to_context, confidence, body } => {
                    if let Some(current) = self.context_manager.current_context() {
                        if current.name != from_context {
                            return Err(format!("Expected context '{}', but was in context '{}'", from_context, current.name).into());
                        }
                    }

                    self.context_manager.pop_context();
                    self.context_manager.push_context(to_context, confidence);
                    let result = self.eval_stmt(*body).await?;
                    Ok(result)
                }
                Stmt::Verify(sources, body) => {
                    // TODO: Implement verification against sources
                    self.eval_stmt(*body).await
                }
                Stmt::Function { name, params: _, body: _, is_async } => {
                    let func = if is_async {
                        let body = Arc::new(move |args: Vec<Value>| -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error>>> + Send>> {
                            Box::pin(async move {
                                Ok(Value::None) // TODO: Implement async function evaluation
                            })
                        });
                        Value::AsyncFn(body)
                    } else {
                        let body = Arc::new(move |_interpreter: &mut Interpreter, _args: Vec<Value>| -> Result<Value, Box<dyn Error>> {
                            Ok(Value::None) // TODO: Implement function evaluation
                        });
                        Value::NativeFunction(body)
                    };
                    self.environment.define(name, func);
                    Ok(Value::None)
                }
            }
        })
    }

    pub fn eval_expr(&mut self, expr: Expr) -> Pin<Box<dyn Future<Output = Result<Value, Box<dyn Error>>> + '_>> {
        Box::pin(async move {
            match expr {
                Expr::Literal(value) => Ok(value),
                Expr::Variable(name) => {
                    self.environment.get(&name)
                        .ok_or_else(|| format!("Undefined variable '{}'.", name).into())
                }
                Expr::Assign(name, value) => {
                    let value = self.eval_expr(*value).await?;
                    self.environment.assign(&name, value.clone())?;
                    Ok(value)
                }
                Expr::Binary(left, operator, right) => {
                    let left_val = self.eval_expr(*left).await?;
                    let right_val = self.eval_expr(*right).await?;
                    match (left_val.clone(), operator.as_str(), right_val.clone()) {
                        (Value::Float(l), "+", Value::Float(r)) => Ok(Value::Float(l + r)),
                        (Value::Float(l), "-", Value::Float(r)) => Ok(Value::Float(l - r)),
                        (Value::Float(l), "*", Value::Float(r)) => Ok(Value::Float(l * r)),
                        (Value::Float(l), "/", Value::Float(r)) => {
                            if r == 0.0 {
                                Err("Division by zero".into())
                            } else {
                                Ok(Value::Float(l / r))
                            }
                        }
                        (Value::Float(l), ">", Value::Float(r)) => Ok(Value::Boolean(l > r)),
                        (Value::Float(l), ">=", Value::Float(r)) => Ok(Value::Boolean(l >= r)),
                        (Value::Float(l), "<", Value::Float(r)) => Ok(Value::Boolean(l < r)),
                        (Value::Float(l), "<=", Value::Float(r)) => Ok(Value::Boolean(l <= r)),
                        (Value::Float(l), "==", Value::Float(r)) => Ok(Value::Boolean(l == r)),
                        (Value::Float(l), "!=", Value::Float(r)) => Ok(Value::Boolean(l != r)),
                        _ => Err(format!("Invalid binary operation: {:?} {} {:?}", left_val, operator, right_val).into()),
                    }
                }
                Expr::Unary(operator, expr) => {
                    let value_val = self.eval_expr(*expr).await?;
                    match (operator.as_str(), value_val.clone()) {
                        ("-", Value::Float(n)) => Ok(Value::Float(-n)),
                        ("!", Value::Boolean(b)) => Ok(Value::Boolean(!b)),
                        _ => Err(format!("Invalid unary operation: {} {:?}", operator, value_val).into()),
                    }
                }
                Expr::Call(callee, arguments) => {
                    let callee = self.eval_expr(*callee).await?;
                    let mut args = Vec::new();
                    for arg in arguments {
                        args.push(self.eval_expr(arg).await?);
                    }
                    match callee {
                        Value::NativeFunction(f) => f(self, args),
                        Value::AsyncFn(f) => f(args).await,
                        _ => Err("Can only call functions.".into()),
                    }
                }
                Expr::Confidence(expr, confidence) => {
                    let value = self.eval_expr(*expr).await?;
                    value.with_confidence(confidence)
                }
                Expr::SemanticMatch(pattern, target) => {
                    let pattern = self.eval_expr(*pattern).await?;
                    let target = self.eval_expr(*target).await?;
                    match (pattern, target) {
                        (Value::String(pattern), Value::String(text)) => {
                            let confidence = self.llm_client.semantic_match(&text, &pattern).await?;
                            Ok(Value::Float(confidence))
                        }
                        _ => Err("Semantic match requires string operands".into()),
                    }
                }
                Expr::Match { value, arms } => {
                    let value = self.eval_expr(*value).await?;
                    for (pattern, (min_conf, max_conf), body) in arms {
                        let pattern_value = self.eval_expr(pattern).await?;
                        if let Some(confidence) = pattern_value.get_confidence() {
                            if confidence >= min_conf && confidence <= max_conf {
                                return self.eval_expr(body).await;
                            }
                        }
                    }
                    Ok(Value::None)
                }
                Expr::UncertainIf { conditions } => {
                    for (condition, confidence_threshold, body) in conditions {
                        let condition_value = self.eval_expr(condition).await?;
                        if let Some(value) = condition_value.as_float() {
                            if value >= confidence_threshold {
                                return self.eval_expr(body).await;
                            }
                        }
                    }
                    Ok(Value::None)
                }
                Expr::TryConfidence { body, threshold, fallback, error_handler } => {
                    let result = self.eval_expr(*body).await?;
                    if let Some(confidence) = result.get_confidence() {
                        if confidence < threshold {
                            return self.eval_expr(*fallback).await;
                        }
                        Ok(result)
                    } else {
                        self.eval_expr(*error_handler).await
                    }
                }
                Expr::Await(expr) => {
                    let value = self.eval_expr(*expr).await?;
                    match value {
                        Value::AsyncFn(f) => f(vec![]).await,
                        _ => Err("Can only await async functions.".into()),
                    }
                }
            }
        })
    }
}