use crate::ast::{Expr, Stmt};
use crate::error::RuntimeError;
use crate::types::Value;
use crate::stdlib::Module;
use std::collections::HashMap;
use std::sync::Arc;
use std::future::Future;
use std::pin::Pin;

pub struct Interpreter {
    environment: HashMap<String, Value>,
    pub output: String,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: HashMap::new(),
            output: String::new(),
        }
    }

    pub fn register_module(&mut self, path: &[&str], module: Module) -> Result<(), RuntimeError> {
        let module_name = path.last().unwrap_or(&"");
        let mut module_obj = Vec::new();
        for (name, value) in module.functions {
            module_obj.push((name.clone(), value.clone()));
        }
        self.environment.insert(module_name.to_string(), Value::Object(module_obj));
        Ok(())
    }

    pub fn interpret<'a>(&'a mut self, statements: Vec<Arc<Stmt>>) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + 'a>> {
        Box::pin(async move {
            let mut last_value = Value::Object(vec![]);
            for statement in statements {
                last_value = self.execute(statement).await?;
            }
            Ok(last_value)
        })
    }

    fn execute<'a>(&'a mut self, statement: Arc<Stmt>) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + 'a>> {
        Box::pin(async move {
            match &*statement {
                Stmt::Expression(expr) => self.evaluate(expr.clone()).await,
                Stmt::Let { name, initializer } => {
                    let value = self.evaluate(initializer.clone()).await?;
                    self.environment.insert(name.clone(), value.clone());
                    Ok(value)
                }
                Stmt::Block(statements) => {
                    let mut last_value = Value::Object(vec![]);
                    for stmt in statements {
                        last_value = self.execute(stmt.clone()).await?;
                    }
                    Ok(last_value)
                }
                Stmt::If { condition, then_branch, else_branch } => {
                    let condition_value = self.evaluate(condition.clone()).await?;
                    if self.is_truthy(&condition_value) {
                        self.execute(then_branch.clone()).await
                    } else if let Some(else_branch) = else_branch {
                        self.execute(else_branch.clone()).await
                    } else {
                        Ok(Value::Object(vec![]))
                    }
                }
                Stmt::While { condition, body } => {
                    let mut last_value = Value::Object(vec![]);
                    loop {
                        let condition_value = self.evaluate(condition.clone()).await?;
                        if !self.is_truthy(&condition_value) {
                            break;
                        }
                        last_value = self.execute(body.clone()).await?;
                    }
                    Ok(last_value)
                }
                Stmt::Function { name, params, body } => {
                    let function = Value::Function {
                        name: name.clone(),
                        params: params.clone(),
                        body: body.clone(),
                        closure: self.environment.clone(),
                    };
                    self.environment.insert(name.clone(), function.clone());
                    Ok(function)
                }
                Stmt::Return(value) => {
                    let value = self.evaluate(value.clone()).await?;
                    Ok(value)
                }
                Stmt::Break => Err(RuntimeError::Break),
                Stmt::Continue => Err(RuntimeError::Continue),
                Stmt::TryCatch { try_block, catch_variable, catch_block } => {
                    match self.execute(try_block.clone()).await {
                        Ok(value) => Ok(value),
                        Err(error) => {
                            let error_value = Value::String(error.to_string());
                            self.environment.insert(catch_variable.clone(), error_value);
                            self.execute(catch_block.clone()).await
                        }
                    }
                }
                Stmt::Throw(expr) => {
                    let value = self.evaluate(expr.clone()).await?;
                    Err(RuntimeError::UserError(value.to_string()))
                }
            }
        })
    }

    fn evaluate<'a>(&'a mut self, expr: Arc<Expr>) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + 'a>> {
        Box::pin(async move {
            match &*expr {
                Expr::Float(n) => Ok(Value::Float(*n)),
                Expr::String(s) => Ok(Value::String(s.clone())),
                Expr::Boolean(b) => Ok(Value::Boolean(*b)),
                Expr::Identifier(name) => {
                    if let Some(value) = self.environment.get(name) {
                        Ok(value.clone())
                    } else {
                        Err(RuntimeError::UndefinedVariable(name.clone()))
                    }
                }
                Expr::Array(elements) => {
                    let mut values = Vec::new();
                    for element in elements {
                        values.push(self.evaluate(element.clone()).await?);
                    }
                    Ok(Value::Array(values))
                }
                Expr::Object(fields) => {
                    let mut values = Vec::new();
                    for (name, value) in fields {
                        values.push((name.clone(), self.evaluate(value.clone()).await?));
                    }
                    Ok(Value::Object(values))
                }
                Expr::Binary { left, operator, right } => {
                    match operator.as_str() {
                        "=" => {
                            let right_val = self.evaluate(right.clone()).await?;
                            match &**left {
                                Expr::Identifier(name) => {
                                    self.environment.insert(name.clone(), right_val.clone());
                                    Ok(right_val)
                                }
                                _ => Err(RuntimeError::TypeError("Invalid assignment target".to_string())),
                            }
                        }
                        "_" => {
                            let module_name = match &**left {
                                Expr::Identifier(name) => name.clone(),
                                _ => return Err(RuntimeError::TypeError("Expected module name".to_string())),
                            };
                            let fn_name = match &**right {
                                Expr::Identifier(name) => name.clone(),
                                _ => return Err(RuntimeError::TypeError("Expected function name".to_string())),
                            };
                            
                            if let Some(Value::Object(module_fns)) = self.environment.get(&module_name) {
                                for (name, value) in module_fns {
                                    if name == &fn_name {
                                        return Ok(value.clone());
                                    }
                                }
                                Err(RuntimeError::UndefinedVariable(format!("Function '{}' not found in module '{}'", fn_name, module_name)))
                            } else {
                                Err(RuntimeError::UndefinedVariable(format!("Module '{}' not found", module_name)))
                            }
                        }
                        _ => {
                            let left = self.evaluate(left.clone()).await?;
                            let right = self.evaluate(right.clone()).await?;
                            match operator.as_str() {
                                "+" => self.add(left, right),
                                "-" => self.subtract(left, right),
                                "*" => self.multiply(left, right),
                                "/" => self.divide(left, right),
                                "==" => Ok(Value::Boolean(left == right)),
                                "!=" => Ok(Value::Boolean(left != right)),
                                "<" => self.less_than(left, right),
                                "<=" => self.less_equal(left, right),
                                ">" => self.greater_than(left, right),
                                ">=" => self.greater_equal(left, right),
                                "." => {
                                    if let Value::Object(fields) = left {
                                        if let Value::String(field_name) = right {
                                            for (name, value) in fields {
                                                if name == field_name {
                                                    return Ok(value);
                                                }
                                            }
                                            Err(RuntimeError::TypeError(format!("Object has no field '{}'", field_name)))
                                        } else {
                                            Err(RuntimeError::TypeError("Expected string as field name".to_string()))
                                        }
                                    } else {
                                        Err(RuntimeError::TypeError("Expected object".to_string()))
                                    }
                                }
                                _ => Err(RuntimeError::TypeError(format!("Unknown operator '{}'", operator))),
                            }
                        }
                    }
                }
                Expr::Unary { operator, operand } => {
                    let value = self.evaluate(operand.clone()).await?;
                    match operator.as_str() {
                        "-" => match value {
                            Value::Float(n) => Ok(Value::Float(-n)),
                            _ => Err(RuntimeError::TypeError("Expected number".to_string())),
                        },
                        "!" => Ok(Value::Boolean(!self.is_truthy(&value))),
                        _ => Err(RuntimeError::TypeError(format!("Unknown operator '{}'", operator))),
                    }
                }
                Expr::Call { function, arguments } => {
                    let callee = self.evaluate(function.clone()).await?;
                    let mut args = Vec::new();
                    for arg in arguments {
                        args.push(self.evaluate(arg.clone()).await?);
                    }
                    self.call(callee, args).await
                }
                Expr::Index { array, index } => {
                    let array = self.evaluate(array.clone()).await?;
                    let index = self.evaluate(index.clone()).await?;
                    match (array, index) {
                        (Value::Array(elements), Value::Float(i)) => {
                            let i = i as usize;
                            if i < elements.len() {
                                Ok(elements[i].clone())
                            } else {
                                Err(RuntimeError::TypeError("Index out of bounds".to_string()))
                            }
                        }
                        _ => Err(RuntimeError::TypeError("Invalid index operation".to_string())),
                    }
                }
            }
        })
    }

    fn call<'a>(&'a mut self, callee: Value, arguments: Vec<Value>) -> Pin<Box<dyn Future<Output = Result<Value, RuntimeError>> + 'a>> {
        Box::pin(async move {
            match callee {
                Value::Function { params, body, closure, .. } => {
                    if params.len() != arguments.len() {
                        return Err(RuntimeError::TypeError(format!(
                            "Expected {} arguments but got {}",
                            params.len(),
                            arguments.len()
                        )));
                    }
                    let old_env = self.environment.clone();
                    self.environment = closure;
                    for (param, arg) in params.iter().zip(arguments) {
                        self.environment.insert(param.clone(), arg);
                    }
                    let result = self.execute(body).await;
                    self.environment = old_env;
                    result
                }
                Value::NativeFunction(f) => f(self, arguments),
                Value::AsyncFn(f) => f(arguments).await,
                _ => Err(RuntimeError::TypeError("Can only call functions".to_string())),
            }
        })
    }

    fn is_truthy(&self, value: &Value) -> bool {
        match value {
            Value::Boolean(b) => *b,
            Value::Float(n) => *n != 0.0,
            Value::String(s) => !s.is_empty(),
            Value::Array(elements) => !elements.is_empty(),
            Value::Object(fields) => !fields.is_empty(),
            Value::Function { .. } => true,
            Value::NativeFunction(_) => true,
            Value::AsyncFn(_) => true,
        }
    }

    fn add(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
            (Value::String(a), Value::String(b)) => Ok(Value::String(format!("{}{}", a, b))),
            _ => Err(RuntimeError::TypeError("Cannot add these types".to_string())),
        }
    }

    fn subtract(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
            _ => Err(RuntimeError::TypeError("Cannot subtract these types".to_string())),
        }
    }

    fn multiply(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
            _ => Err(RuntimeError::TypeError("Cannot multiply these types".to_string())),
        }
    }

    fn divide(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => {
                if b == 0.0 {
                    Err(RuntimeError::DivisionByZero)
                } else {
                    Ok(Value::Float(a / b))
                }
            }
            _ => Err(RuntimeError::TypeError("Cannot divide these types".to_string())),
        }
    }

    fn less_than(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a < b)),
            _ => Err(RuntimeError::TypeError("Cannot compare these types".to_string())),
        }
    }

    fn less_equal(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a <= b)),
            _ => Err(RuntimeError::TypeError("Cannot compare these types".to_string())),
        }
    }

    fn greater_than(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a > b)),
            _ => Err(RuntimeError::TypeError("Cannot compare these types".to_string())),
        }
    }

    fn greater_equal(&self, left: Value, right: Value) -> Result<Value, RuntimeError> {
        match (left, right) {
            (Value::Float(a), Value::Float(b)) => Ok(Value::Boolean(a >= b)),
            _ => Err(RuntimeError::TypeError("Cannot compare these types".to_string())),
        }
    }
}