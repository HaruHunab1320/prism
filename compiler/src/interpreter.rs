use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use crate::ast::{Expr, Stmt};
use crate::environment::Environment;
use crate::error::RuntimeError;
use crate::types::Value;

pub struct Interpreter {
    pub environment: Environment,
    pub metrics: Metrics,
}

pub struct Metrics {
    pub total_diagnoses: f64,
    pub correct_diagnoses: f64,
    pub false_positives: f64,
    pub false_negatives: f64,
    pub confidence_sum: f64,
    pub execution_time: f64,
}

impl Metrics {
    pub fn new() -> Self {
        Self {
            total_diagnoses: 0.0,
            correct_diagnoses: 0.0,
            false_positives: 0.0,
            false_negatives: 0.0,
            confidence_sum: 0.0,
            execution_time: 0.0,
        }
    }

    pub fn accuracy(&self) -> f64 {
        if self.total_diagnoses == 0.0 {
            0.0
        } else {
            self.correct_diagnoses / self.total_diagnoses
        }
    }

    pub fn precision(&self) -> f64 {
        if self.correct_diagnoses + self.false_positives == 0.0 {
            0.0
        } else {
            self.correct_diagnoses / (self.correct_diagnoses + self.false_positives)
        }
    }

    pub fn recall(&self) -> f64 {
        if self.correct_diagnoses + self.false_negatives == 0.0 {
            0.0
        } else {
            self.correct_diagnoses / (self.correct_diagnoses + self.false_negatives)
        }
    }

    pub fn f1_score(&self) -> f64 {
        let precision = self.precision();
        let recall = self.recall();
        if precision + recall == 0.0 {
            0.0
        } else {
            2.0 * (precision * recall) / (precision + recall)
        }
    }

    pub fn average_confidence(&self) -> f64 {
        if self.total_diagnoses == 0.0 {
            0.0
        } else {
            self.confidence_sum / self.total_diagnoses
        }
    }
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
            metrics: Metrics::new(),
        }
    }

    pub fn register_native_function<F>(&mut self, name: &str, f: F)
    where
        F: Fn(&mut Interpreter, Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + Send>> + Send + Sync + 'static,
    {
        self.environment.define(
            name,
            Value::NativeFunction(Arc::new(f)),
        );
    }

    pub fn eval_expr<'a>(&'a mut self, expr: &'a Expr) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + Send + 'a>> {
        Box::pin(async move {
            match expr {
                Expr::Float(n) => Ok(Value::Float(*n)),
                Expr::String(s) => Ok(Value::String(s.clone())),
                Expr::Boolean(b) => Ok(Value::Boolean(*b)),
                Expr::Identifier(name) => {
                    self.environment.get(name).ok_or_else(|| RuntimeError::UndefinedVariable(name.clone()))
                }
                Expr::Array(elements) => {
                    let mut values = Vec::new();
                    for element in elements {
                        values.push(self.eval_expr(element).await?);
                    }
                    Ok(Value::Array(values))
                }
                Expr::Object(fields) => {
                    let mut values = Vec::new();
                    for (name, value) in fields {
                        values.push((name.clone(), self.eval_expr(value).await?));
                    }
                    Ok(Value::Object(values))
                }
                Expr::Binary { left, operator, right } => {
                    let left = self.eval_expr(left).await?;
                    let right = self.eval_expr(right).await?;
                    match operator.as_str() {
                        "+" => left.add(&right),
                        "-" => left.subtract(&right),
                        "*" => left.multiply(&right),
                        "/" => left.divide(&right),
                        "==" => left.equals(&right),
                        "!=" => left.not_equals(&right),
                        "<" => left.less_than(&right),
                        "<=" => left.less_than_or_equal(&right),
                        ">" => left.greater_than(&right),
                        ">=" => left.greater_than_or_equal(&right),
                        "&&" => left.and(&right),
                        "||" => left.or(&right),
                        "." => left.get_property(&right),
                        "~>" => left.with_confidence(&right),
                        _ => Err(RuntimeError::TypeError(format!("Unknown operator '{}'", operator))),
                    }
                }
                Expr::Unary { operator, operand } => {
                    let value = self.eval_expr(operand).await?;
                    match operator.as_str() {
                        "-" => value.negate(),
                        "!" => value.not(),
                        _ => Err(RuntimeError::TypeError(format!("Unknown operator '{}'", operator))),
                    }
                }
                Expr::Call { function, arguments } => {
                    let callee = self.eval_expr(function).await?;
                    let mut args = Vec::new();
                    for arg in arguments {
                        args.push(self.eval_expr(arg).await?);
                    }
                    callee.call(self, args).await
                }
                Expr::Index { array, index } => {
                    let array = self.eval_expr(array).await?;
                    let index = self.eval_expr(index).await?;
                    array.get_index(&index)
                }
            }
        })
    }

    pub fn eval_stmt<'a>(&'a mut self, stmt: &'a Stmt) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + Send + 'a>> {
        Box::pin(async move {
            match stmt {
                Stmt::Expression(expr) => self.eval_expr(expr).await,
                Stmt::Let { name, initializer } => {
                    let value = self.eval_expr(initializer).await?;
                    self.environment.define(name, value.clone());
                    Ok(Value::Object(vec![]))
                }
                Stmt::Block(statements) => {
                    let mut result = Value::Object(vec![]);
                    for stmt in statements {
                        result = self.eval_stmt(stmt).await?;
                    }
                    Ok(result)
                }
                Stmt::If { condition, then_branch, else_branch } => {
                    let condition = self.eval_expr(condition).await?;
                    match condition {
                        Value::Boolean(true) => self.eval_stmt(then_branch).await,
                        Value::Boolean(false) => {
                            if let Some(else_branch) = else_branch {
                                self.eval_stmt(else_branch).await
                            } else {
                                Ok(Value::Object(vec![]))
                            }
                        }
                        _ => Err(RuntimeError::TypeError("Condition must be a boolean".to_string())),
                    }
                }
                Stmt::While { condition, body } => {
                    let mut result = Value::Object(vec![]);
                    while let Value::Boolean(true) = self.eval_expr(condition).await? {
                        result = self.eval_stmt(body).await?;
                    }
                    Ok(result)
                }
                Stmt::Function { name, params, body } => {
                    let params = params.clone();
                    let body = body.clone();
                    let env = self.environment.clone();
                    let func = Value::AsyncFn(Arc::new(move |args: Vec<Value>| {
                        let params = params.clone();
                        let body = body.clone();
                        let env = env.clone();
                        Box::pin(async move {
                            if args.len() != params.len() {
                                return Err(RuntimeError::TypeError(format!(
                                    "Expected {} arguments but got {}",
                                    params.len(),
                                    args.len()
                                )));
                            }
                            let mut interpreter = Interpreter {
                                environment: env.clone(),
                                metrics: Metrics::new(),
                            };
                            for (param, arg) in params.iter().zip(args) {
                                interpreter.environment.define(param, arg);
                            }
                            match interpreter.eval_stmt(&body).await {
                                Ok(value) => Ok(value),
                                Err(RuntimeError::Return(value)) => Ok(value),
                                Err(e) => Err(e),
                            }
                        })
                    }));
                    self.environment.define(name, func);
                    Ok(Value::Object(vec![]))
                }
                Stmt::Return(expr) => {
                    let value = self.eval_expr(expr).await?;
                    Err(RuntimeError::Return(value))
                }
                Stmt::Break => Err(RuntimeError::Break),
                Stmt::Continue => Err(RuntimeError::Continue),
                Stmt::TryCatch { try_block, catch_variable, catch_block } => {
                    match self.eval_stmt(try_block).await {
                        Ok(value) => Ok(value),
                        Err(RuntimeError::Throw(value)) => {
                            self.environment.define(catch_variable, value);
                            self.eval_stmt(catch_block).await
                        }
                        Err(e) => Err(e),
                    }
                }
                Stmt::Throw(expr) => {
                    let value = self.eval_expr(expr).await?;
                    Err(RuntimeError::Throw(value))
                }
            }
        })
    }
}