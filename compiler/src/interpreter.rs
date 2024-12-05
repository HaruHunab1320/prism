use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::sync::Mutex;
use crate::ast::{Value, Expr, Stmt, UnaryOp, BinaryOp, AsyncResult};
use crate::lexer::Lexer;
use crate::parser::Parser;
use crate::environment::Environment;

#[derive(Clone)]
pub struct Interpreter {
    api_key: String,
    environment: Arc<Mutex<Environment>>,
}

impl Interpreter {
    pub fn new(api_key: String) -> Self {
        let interpreter = Self {
            api_key,
            environment: Arc::new(Mutex::new(Environment::new())),
        };
        
        // Register standard library functions after creation
        let i = interpreter.clone();
        crate::stdlib::register_all_functions(&i);
        
        interpreter
    }

    pub fn get_api_key(&self) -> &str {
        &self.api_key
    }

    pub fn eval(&self, source: String) -> AsyncResult<Value> {
        let interpreter = self.clone();
        
        Box::pin(async move {
            // 1. Lexical analysis
            let mut lexer = Lexer::new(&source);
            let tokens = lexer.lex()?;

            // 2. Parsing
            let mut parser = Parser::new(tokens);
            let ast = parser.parse()?;

            // 3. Evaluation
            let mut result = Value::Null;
            for stmt in ast {
                result = interpreter.eval_statement(stmt).await?;
            }

            Ok(result)
        })
    }

    pub fn register_native_function<F>(&self, name: &str, f: F)
    where
        F: Fn(&Interpreter, Vec<Value>) -> AsyncResult<Value> + Send + Sync + 'static,
    {
        if let Ok(mut env) = self.environment.lock() {
            env.define(name.to_string(), Value::AsyncFn(Arc::new(f)));
        }
    }

    fn eval_statement(&self, stmt: Stmt) -> AsyncResult<Value> {
        let interpreter = self.clone();
        
        Box::pin(async move {
            match stmt {
                Stmt::Expression(expr) => interpreter.eval_expression(expr).await,
                Stmt::Let(name, expr) => {
                    let value = interpreter.eval_expression(expr).await?;
                    if let Ok(mut env) = interpreter.environment.lock() {
                        env.define(name, value.clone());
                    }
                    Ok(value)
                },
                Stmt::Block(statements) => {
                    if let Ok(mut env) = interpreter.environment.lock() {
                        env.push();
                    }
                    let mut result = Value::Null;
                    for stmt in statements {
                        result = interpreter.eval_statement(stmt).await?;
                    }
                    if let Ok(mut env) = interpreter.environment.lock() {
                        env.pop();
                    }
                    Ok(result)
                },
                Stmt::If { condition, then_branch, else_branch } => {
                    let cond_value = interpreter.eval_expression(condition).await?;
                    if let Value::Bool(true) = cond_value {
                        interpreter.eval_statement(*then_branch).await
                    } else if let Some(else_stmt) = else_branch {
                        interpreter.eval_statement(*else_stmt).await
                    } else {
                        Ok(Value::Null)
                    }
                },
                Stmt::While { condition, body } => {
                    loop {
                        let cond_value = interpreter.eval_expression(condition.clone()).await?;
                        if let Value::Bool(true) = cond_value {
                            interpreter.eval_statement(*body.clone()).await?;
                        } else {
                            break;
                        }
                    }
                    Ok(Value::Null)
                },
                Stmt::Function { name, params, body, is_async: _ } => {
                    let func_interpreter = interpreter.clone();
                    let func = Value::AsyncFn(Arc::new(move |_, args: Vec<Value>| {
                        let params = params.clone();
                        let body = body.clone();
                        let interpreter = func_interpreter.clone();
                        
                        Box::pin(async move {
                            if args.len() != params.len() {
                                return Err(format!("Expected {} arguments but got {}", params.len(), args.len()).into());
                            }

                            let mut new_env = Environment::new();
                            if let Ok(env) = interpreter.environment.lock() {
                                new_env = (*env).clone();
                            }
                            new_env.push();
                            for (param, arg) in params.iter().zip(args) {
                                new_env.define(param.clone(), arg);
                            }

                            let func_interpreter = Interpreter {
                                api_key: interpreter.api_key.clone(),
                                environment: Arc::new(Mutex::new(new_env)),
                            };

                            let result = func_interpreter.eval_statement(*body).await;
                            if let Ok(mut env) = func_interpreter.environment.lock() {
                                env.pop();
                            }
                            result
                        })
                    }));
                    if let Ok(mut env) = interpreter.environment.lock() {
                        env.define(name, func.clone());
                    }
                    Ok(func)
                },
                Stmt::Return(expr) => interpreter.eval_expression(*expr).await,
            }
        })
    }

    fn eval_expression(&self, expr: Expr) -> AsyncResult<Value> {
        let interpreter = self.clone();
        
        Box::pin(async move {
            match expr {
                Expr::Literal(value) => Ok(value),
                Expr::Variable(name) => {
                    if let Ok(env) = interpreter.environment.lock() {
                        if let Some(value) = env.get(&name) {
                            Ok(value)
                        } else {
                            Err(format!("Undefined variable '{}'", name).into())
                        }
                    } else {
                        Err("Failed to access environment".into())
                    }
                },
                Expr::Unary { operator, expr } => {
                    let value = interpreter.eval_expression(*expr).await?;
                    match operator {
                        UnaryOp::Not => {
                            if let Value::Bool(b) = value {
                                Ok(Value::Bool(!b))
                            } else {
                                Err("Operand must be a boolean".into())
                            }
                        },
                        UnaryOp::Minus => {
                            if let Value::Number(n) = value {
                                Ok(Value::Number(-n))
                            } else {
                                Err("Operand must be a number".into())
                            }
                        },
                    }
                },
                Expr::Binary { left, operator, right } => {
                    let left_val = interpreter.eval_expression(*left).await?;
                    let right_val = interpreter.eval_expression(*right).await?;
                    
                    match operator {
                        BinaryOp::Add => match (&left_val, &right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a + b)),
                            (Value::String(a), Value::String(b)) => Ok(Value::String(a.clone() + b)),
                            _ => Err("Operands must be two numbers or two strings".into()),
                        },
                        BinaryOp::Subtract => match (left_val, right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a - b)),
                            _ => Err("Operands must be numbers".into()),
                        },
                        BinaryOp::Multiply => match (left_val, right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a * b)),
                            _ => Err("Operands must be numbers".into()),
                        },
                        BinaryOp::Divide => match (left_val, right_val) {
                            (Value::Number(_), Value::Number(b)) if b == 0.0 => {
                                Err("Division by zero".into())
                            },
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Number(a / b)),
                            _ => Err("Operands must be numbers".into()),
                        },
                        BinaryOp::Equal => Ok(Value::Bool(left_val == right_val)),
                        BinaryOp::NotEqual => Ok(Value::Bool(left_val != right_val)),
                        BinaryOp::Greater => match (left_val, right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a > b)),
                            _ => Err("Operands must be numbers".into()),
                        },
                        BinaryOp::GreaterEqual => match (left_val, right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a >= b)),
                            _ => Err("Operands must be numbers".into()),
                        },
                        BinaryOp::Less => match (left_val, right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a < b)),
                            _ => Err("Operands must be numbers".into()),
                        },
                        BinaryOp::LessEqual => match (left_val, right_val) {
                            (Value::Number(a), Value::Number(b)) => Ok(Value::Bool(a <= b)),
                            _ => Err("Operands must be numbers".into()),
                        },
                        BinaryOp::And => match (left_val, right_val) {
                            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a && b)),
                            _ => Err("Operands must be booleans".into()),
                        },
                        BinaryOp::Or => match (left_val, right_val) {
                            (Value::Bool(a), Value::Bool(b)) => Ok(Value::Bool(a || b)),
                            _ => Err("Operands must be booleans".into()),
                        },
                    }
                },
                Expr::Call { callee, arguments } => {
                    let callee_val = interpreter.eval_expression(*callee).await?;
                    let mut evaluated_args = Vec::new();
                    
                    for arg in arguments {
                        evaluated_args.push(interpreter.eval_expression(arg).await?);
                    }
                    
                    match callee_val {
                        Value::AsyncFn(func) => {
                            func(&interpreter, evaluated_args).await
                        },
                        _ => Err("Can only call functions".into()),
                    }
                },
                Expr::Match { value, arms } => {
                    let match_value = interpreter.eval_expression(*value).await?;
                    for (pattern, result) in arms {
                        let pattern_value = interpreter.eval_expression(pattern).await?;
                        if interpreter.pattern_matches(&match_value, &pattern_value).await? {
                            return interpreter.eval_expression(*result).await;
                        }
                    }
                    Ok(Value::Null)
                },
                Expr::TryConfidence { body, threshold, fallback, error_handler } => {
                    let result = interpreter.eval_expression(*body).await;
                    match result {
                        Ok(value) => {
                            if let Some(confidence) = value.get_confidence() {
                                if confidence >= threshold {
                                    Ok(value)
                                } else {
                                    interpreter.eval_expression(*fallback).await
                                }
                            } else {
                                interpreter.eval_expression(*fallback).await
                            }
                        },
                        Err(e) => {
                            if let Some(handler) = error_handler {
                                interpreter.eval_expression(*handler).await
                            } else {
                                Err(e)
                            }
                        }
                    }
                },
                Expr::Verify { value, pattern } => {
                    let val = interpreter.eval_expression(*value).await?;
                    let pat = interpreter.eval_expression(*pattern).await?;
                    
                    // Perform semantic verification
                    if let (Value::String(v), Value::String(p)) = (&val, &pat) {
                        let client = crate::stdlib::llm::LLMClient::new(interpreter.api_key.clone());
                        let confidence = client.semantic_match(v, p).await?;
                        Ok(Value::Number(confidence))
                    } else {
                        Err("Verify requires string arguments".into())
                    }
                },
            }
        })
    }

    async fn pattern_matches(&self, value: &Value, pattern: &Value) -> Result<bool, Box<dyn Error + Send + Sync>> {
        let interpreter = self.clone();
        
        Box::pin(async move {
            match (value, pattern) {
                // Direct value matching
                (Value::Number(n1), Value::Number(n2)) => Ok((n1 - n2).abs() < f64::EPSILON),
                (Value::String(s1), Value::String(s2)) => Ok(s1 == s2),
                (Value::Bool(b1), Value::Bool(b2)) => Ok(b1 == b2),
                (Value::Null, Value::Null) => Ok(true),
                
                // Semantic matching for strings
                (Value::String(text), Value::Pattern(pattern)) => {
                    let client = crate::stdlib::llm::LLMClient::new(interpreter.api_key.clone());
                    let similarity = client.semantic_match(text, pattern).await?;
                    Ok(similarity >= 0.8)
                },
                
                // Array pattern matching
                (Value::Array(arr1), Value::Array(arr2)) => {
                    if arr1.len() != arr2.len() {
                        return Ok(false);
                    }
                    for (v1, v2) in arr1.iter().zip(arr2.iter()) {
                        if !interpreter.pattern_matches(v1, v2).await? {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                },
                
                // Object pattern matching
                (Value::Object(obj1), Value::Object(obj2)) => {
                    for (key, pattern_value) in obj2 {
                        if let Some(obj_value) = obj1.get(key) {
                            if !interpreter.pattern_matches(obj_value, pattern_value).await? {
                                return Ok(false);
                            }
                        } else {
                            return Ok(false);
                        }
                    }
                    Ok(true)
                },
                
                // Wildcard pattern
                (_, Value::Wildcard) => Ok(true),
                
                // No match for different types
                _ => Ok(false),
            }
        }).await
    }
}
