use crate::ast::{Expr, Stmt};
use crate::error::Error;
use crate::value::{Value, ValueKind};
use crate::environment::Environment;
use crate::lexer::TokenType;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

type AsyncResult<'a, T> = Pin<Box<dyn Future<Output = Result<T, Box<dyn std::error::Error + Send + Sync + 'a>>> + Send + 'a>>;

#[derive(Clone)]
pub struct Interpreter {
    environment: Arc<Environment>,
    api_key: String,
}

impl Interpreter {
    pub fn new(api_key: String) -> Arc<Self> {
        Arc::new(Self {
            environment: Arc::new(Environment::new()),
            api_key,
        })
    }

    pub fn with_environment(environment: Arc<Environment>, api_key: String) -> Arc<Self> {
        Arc::new(Self {
            environment,
            api_key,
        })
    }

    pub fn eval<'a>(self: &'a Arc<Self>, source: String) -> AsyncResult<'a, Value> {
        Box::pin(async move {
            let mut lexer = crate::lexer::Lexer::new(&source);
            let tokens = lexer.scan_tokens()?;
            let mut parser = crate::parser::Parser::new(tokens);
            let statements = parser.parse()?;
            
            let mut last_value = Value::new(ValueKind::Null);
            for stmt in statements {
                last_value = self.execute(stmt).await?;
            }
            Ok(last_value)
        })
    }

    fn execute<'a>(self: &'a Arc<Self>, stmt: Stmt) -> AsyncResult<'a, Value> {
        Box::pin(async move {
            match stmt {
                Stmt::Expression(expr) => self.evaluate(*expr).await,
                Stmt::Let(name, initializer) => {
                    let value = match initializer {
                        Some(expr) => self.evaluate(*expr).await?,
                        None => Value::new(ValueKind::Null),
                    };
                    self.environment.define(name, value.clone()).await;
                    Ok(value)
                },
                Stmt::Block(statements) => {
                    let environment = Arc::new(Environment::with_enclosing(self.environment.clone()));
                    let interpreter = Arc::new(Self {
                        environment,
                        api_key: self.api_key.clone(),
                    });
                    
                    let mut last_value = Value::new(ValueKind::Null);
                    for stmt in statements {
                        last_value = interpreter.execute(stmt).await?;
                    }
                    Ok(last_value)
                },
                Stmt::If { condition, then_branch, else_branch } => {
                    let condition_value = self.evaluate(*condition).await?;
                    if is_truthy(&condition_value) {
                        self.execute(*then_branch).await
                    } else if let Some(else_branch) = else_branch {
                        self.execute(*else_branch).await
                    } else {
                        Ok(Value::new(ValueKind::Null))
                    }
                },
                Stmt::UncertainIf { condition, then_branch, medium_branch, low_branch } => {
                    let condition_value = self.evaluate(*condition).await?;
                    let confidence = condition_value.confidence;
                    
                    if confidence > 0.8 {
                        self.execute(*then_branch).await
                    } else if confidence > 0.5 {
                        if let Some(medium_branch) = medium_branch {
                            self.execute(*medium_branch).await
                        } else {
                            Ok(Value::new(ValueKind::Null))
                        }
                    } else {
                        if let Some(low_branch) = low_branch {
                            self.execute(*low_branch).await
                        } else {
                            Ok(Value::new(ValueKind::Null))
                        }
                    }
                },
                Stmt::While { condition, body } => {
                    let mut last_value = Value::new(ValueKind::Null);
                    while is_truthy(&self.evaluate(*condition.clone()).await?) {
                        last_value = self.execute(*body.clone()).await?;
                    }
                    Ok(last_value)
                },
                Stmt::Function { name, params, body, is_async, confidence } => {
                    let function = Value::new(ValueKind::Function {
                        name: name.clone(),
                        params,
                        body: *body,
                        closure: self.environment.clone(),
                        is_async,
                    });
                    
                    let function = if let Some(conf) = confidence {
                        Value::with_confidence(function.kind, conf)
                    } else {
                        function
                    };
                    
                    self.environment.define(name, function.clone()).await;
                    Ok(function)
                },
                Stmt::Return(value) => {
                    let value = match value {
                        Some(expr) => self.evaluate(*expr).await?,
                        None => Value::new(ValueKind::Null),
                    };
                    Ok(value)
                },
                Stmt::Context { name, body } => {
                    let result = self.execute(*body).await?;
                    Ok(Value::in_context(result.kind, name))
                },
            }
        })
    }

    fn evaluate<'a>(self: &'a Arc<Self>, expr: Expr) -> AsyncResult<'a, Value> {
        Box::pin(async move {
            match expr {
                Expr::Literal(value) => Ok(value),
                Expr::Variable(name) => self.environment.get(&name).await,
                Expr::Assign { name, value } => {
                    let value = self.evaluate(*value).await?;
                    self.environment.assign(&name, value.clone()).await?;
                    Ok(value)
                },
                Expr::Binary { left, operator, right } => {
                    let left = self.evaluate(*left).await?;
                    let right = self.evaluate(*right).await?;
                    self.evaluate_binary_op(left, operator.token_type, right)
                },
                Expr::Unary { operator, right } => {
                    let right = self.evaluate(*right).await?;
                    self.evaluate_unary_op(operator.token_type, right)
                },
                Expr::Call { callee, arguments } => {
                    let callee = self.evaluate(*callee).await?;
                    let mut args = Vec::new();
                    for arg in arguments {
                        args.push(self.evaluate(arg).await?);
                    }
                    self.call_function(callee, args).await
                },
                Expr::Logical { left, operator, right } => {
                    let left = self.evaluate(*left).await?;
                    
                    match operator.token_type {
                        TokenType::And => {
                            if !is_truthy(&left) {
                                Ok(left)
                            } else {
                                self.evaluate(*right).await
                            }
                        },
                        TokenType::Or => {
                            if is_truthy(&left) {
                                Ok(left)
                            } else {
                                self.evaluate(*right).await
                            }
                        },
                        _ => Err(Box::new(Error::new("Invalid logical operator")) as Box<dyn std::error::Error + Send + Sync>),
                    }
                },
                Expr::Confidence { expr, confidence } => {
                    let value = self.evaluate(*expr).await?;
                    Ok(Value::with_confidence(value.kind, confidence))
                },
                Expr::ConfidenceCombine { left, right } => {
                    let left = self.evaluate(*left).await?;
                    let right = self.evaluate(*right).await?;
                    Ok(left.combine_confidence(right.confidence))
                },
                Expr::InContext { context, body } => {
                    let value = self.evaluate(*body).await?;
                    Ok(Value::in_context(value.kind, context))
                },
                Expr::Get { object, name: _ } => {
                    let _object = self.evaluate(*object).await?;
                    // TODO: Implement property access
                    Err(Box::new(Error::new("Property access not implemented")) as Box<dyn std::error::Error + Send + Sync>)
                },
                Expr::Grouping(expr) => self.evaluate(*expr).await,
            }
        })
    }

    fn evaluate_binary_op(&self, left: Value, operator: TokenType, right: Value) 
        -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        use TokenType::*;
        
        match (&left.kind, operator, &right.kind) {
            (ValueKind::Number(a), Plus, ValueKind::Number(b)) => 
                Ok(Value::new(ValueKind::Number(a + b))),
            (ValueKind::Number(a), Minus, ValueKind::Number(b)) => 
                Ok(Value::new(ValueKind::Number(a - b))),
            (ValueKind::Number(a), Star, ValueKind::Number(b)) => 
                Ok(Value::new(ValueKind::Number(a * b))),
            (ValueKind::Number(a), Slash, ValueKind::Number(b)) => {
                if *b == 0.0 {
                    Err(Box::new(Error::new("Division by zero")))
                } else {
                    Ok(Value::new(ValueKind::Number(a / b)))
                }
            },
            (ValueKind::String(a), Plus, ValueKind::String(b)) => 
                Ok(Value::new(ValueKind::String(format!("{}{}", a, b)))),
            (ValueKind::Number(a), Greater, ValueKind::Number(b)) => 
                Ok(Value::new(ValueKind::Bool(a > b))),
            (ValueKind::Number(a), GreaterEqual, ValueKind::Number(b)) => 
                Ok(Value::new(ValueKind::Bool(a >= b))),
            (ValueKind::Number(a), Less, ValueKind::Number(b)) => 
                Ok(Value::new(ValueKind::Bool(a < b))),
            (ValueKind::Number(a), LessEqual, ValueKind::Number(b)) => 
                Ok(Value::new(ValueKind::Bool(a <= b))),
            (ValueKind::Number(a), EqualEqual, ValueKind::Number(b)) => 
                Ok(Value::new(ValueKind::Bool((a - b).abs() < f64::EPSILON))),
            (ValueKind::String(a), EqualEqual, ValueKind::String(b)) => 
                Ok(Value::new(ValueKind::Bool(a == b))),
            (ValueKind::Bool(a), EqualEqual, ValueKind::Bool(b)) => 
                Ok(Value::new(ValueKind::Bool(a == b))),
            (ValueKind::Null, EqualEqual, ValueKind::Null) => 
                Ok(Value::new(ValueKind::Bool(true))),
            _ => Err(Box::new(Error::new("Invalid operands for binary operator"))),
        }
    }

    fn evaluate_unary_op(&self, operator: TokenType, right: Value) 
        -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        use TokenType::*;
        
        match (operator, &right.kind) {
            (Minus, ValueKind::Number(n)) => Ok(Value::new(ValueKind::Number(-n))),
            (Bang, _) => Ok(Value::new(ValueKind::Bool(!is_truthy(&right)))),
            _ => Err(Box::new(Error::new("Invalid operand for unary operator"))),
        }
    }

    fn call_function<'a>(self: &'a Arc<Self>, callee: Value, arguments: Vec<Value>) -> AsyncResult<'a, Value> {
        Box::pin(async move {
            match callee.kind {
                ValueKind::Function { params, body, closure, is_async, .. } => {
                    let environment = Arc::new(Environment::with_enclosing(closure));
                    
                    for (param, arg) in params.iter().zip(arguments) {
                        environment.define(param.clone(), arg).await;
                    }
                    
                    let interpreter = Arc::new(Self {
                        environment,
                        api_key: self.api_key.clone(),
                    });
                    
                    if is_async {
                        interpreter.execute(body).await
                    } else {
                        interpreter.execute(body).await
                    }
                },
                _ => Err(Box::new(Error::new("Can only call functions")) as Box<dyn std::error::Error + Send + Sync>),
            }
        })
    }
}

fn is_truthy(value: &Value) -> bool {
    match &value.kind {
        ValueKind::Bool(b) => *b,
        ValueKind::Null => false,
        _ => true,
    }
}
