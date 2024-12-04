use crate::ast::{Expr, Stmt};
use crate::types::{Value, RuntimeError};
use std::collections::HashMap;

pub struct Environment {
    scopes: Vec<HashMap<String, Value>>,
}

impl Environment {
    pub fn new() -> Self {
        let mut env = Self {
            scopes: Vec::new(),
        };
        env.push_scope();
        env
    }

    pub fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    pub fn define(&mut self, name: String, value: Value) {
        if let Some(scope) = self.scopes.last_mut() {
            scope.insert(name, value);
        }
    }

    pub fn assign(&mut self, name: &str, value: Value) -> Result<(), RuntimeError> {
        for scope in self.scopes.iter_mut().rev() {
            if scope.contains_key(name) {
                scope.insert(name.to_string(), value);
                return Ok(());
            }
        }
        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }

    pub fn get(&self, name: &str) -> Result<Value, RuntimeError> {
        for scope in self.scopes.iter().rev() {
            if let Some(value) = scope.get(name) {
                return Ok(value.clone());
            }
        }
        Err(RuntimeError::UndefinedVariable(name.to_string()))
    }
}

pub struct Interpreter {
    environment: Environment,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            environment: Environment::new(),
        }
    }

    pub fn eval_stmt(&mut self, stmt: &Stmt) -> Result<Value, RuntimeError> {
        match stmt {
            Stmt::Expression(expr) => self.eval_expr(expr),
            Stmt::Let { name, initializer } => {
                let value = self.eval_expr(initializer)?;
                self.environment.define(name.clone(), value.clone());
                Ok(value)
            }
            Stmt::Block(statements) => {
                self.environment.push_scope();
                let mut result = Value::Void;
                for stmt in statements {
                    result = self.eval_stmt(stmt)?;
                }
                self.environment.pop_scope();
                Ok(result)
            }
            Stmt::If { condition, then_branch, else_branch } => {
                let cond = self.eval_expr(condition)?;
                match cond {
                    Value::Boolean(true) => self.eval_stmt(then_branch),
                    Value::Boolean(false) => {
                        if let Some(else_branch) = else_branch {
                            self.eval_stmt(else_branch)
                        } else {
                            Ok(Value::Void)
                        }
                    }
                    _ => Err(RuntimeError::TypeError("Condition must be a boolean".to_string())),
                }
            }
            Stmt::While { condition, body } => {
                while let Value::Boolean(true) = self.eval_expr(condition)? {
                    match self.eval_stmt(body) {
                        Err(RuntimeError::Break) => break,
                        Err(RuntimeError::Continue) => continue,
                        Err(e) => return Err(e),
                        Ok(_) => (),
                    }
                }
                Ok(Value::Void)
            }
            Stmt::Break => Err(RuntimeError::Break),
            Stmt::Continue => Err(RuntimeError::Continue),
            Stmt::TryCatch { try_block, catch_variable, catch_block } => {
                let try_result = self.eval_stmt(&Stmt::Block(try_block.clone()));
                match try_result {
                    Ok(value) => Ok(value),
                    Err(RuntimeError::Throw(value)) => {
                        self.environment.push_scope();
                        self.environment.define(catch_variable.clone(), value);
                        let result = self.eval_stmt(&Stmt::Block(catch_block.clone()));
                        self.environment.pop_scope();
                        result
                    }
                    Err(e) => Err(e),
                }
            }
            Stmt::Function { name, params, body } => {
                self.environment.define(name.clone(), Value::Function(params.clone(), body.clone()));
                Ok(Value::Void)
            }
            Stmt::Return(expr) => {
                let value = self.eval_expr(expr)?;
                Err(RuntimeError::Return(value))
            }
            Stmt::Throw(expr) => {
                let value = self.eval_expr(expr)?;
                Err(RuntimeError::Throw(value))
            }
        }
    }

    pub fn eval_expr(&mut self, expr: &Expr) -> Result<Value, RuntimeError> {
        match expr {
            Expr::Float(n) => Ok(Value::Float(*n)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Boolean(b) => Ok(Value::Boolean(*b)),
            Expr::Identifier(name) => self.environment.get(name),
            Expr::Array(elements) => {
                let mut values = Vec::new();
                for element in elements {
                    values.push(self.eval_expr(element)?);
                }
                Ok(Value::Array(values))
            }
            Expr::Binary { left, operator, right } => {
                let left = self.eval_expr(left)?;
                let right = self.eval_expr(right)?;
                match (left.clone(), operator.as_str(), right.clone()) {
                    (Value::Float(a), "+", Value::Float(b)) => Ok(Value::Float(a + b)),
                    (Value::Float(a), "-", Value::Float(b)) => Ok(Value::Float(a - b)),
                    (Value::Float(a), "*", Value::Float(b)) => Ok(Value::Float(a * b)),
                    (Value::Float(a), "/", Value::Float(b)) => {
                        if b == 0.0 {
                            Err(RuntimeError::DivisionByZero)
                        } else {
                            Ok(Value::Float(a / b))
                        }
                    }
                    (Value::Float(a), "==", Value::Float(b)) => Ok(Value::Boolean(a == b)),
                    (Value::Float(a), "!=", Value::Float(b)) => Ok(Value::Boolean(a != b)),
                    (Value::Float(a), "<", Value::Float(b)) => Ok(Value::Boolean(a < b)),
                    (Value::Float(a), "<=", Value::Float(b)) => Ok(Value::Boolean(a <= b)),
                    (Value::Float(a), ">", Value::Float(b)) => Ok(Value::Boolean(a > b)),
                    (Value::Float(a), ">=", Value::Float(b)) => Ok(Value::Boolean(a >= b)),
                    (Value::Boolean(a), "&&", Value::Boolean(b)) => Ok(Value::Boolean(a && b)),
                    (Value::Boolean(a), "||", Value::Boolean(b)) => Ok(Value::Boolean(a || b)),
                    (Value::String(a), "+", Value::String(b)) => Ok(Value::String(a + &b)),
                    (Value::Float(a), "~>", Value::Float(b)) => {
                        if b < 0.0 || b > 1.0 {
                            Err(RuntimeError::TypeError("Confidence value must be between 0 and 1".to_string()))
                        } else {
                            Ok(Value::Float(a * b))
                        }
                    }
                    _ => Err(RuntimeError::TypeError(format!(
                        "Invalid operation: {:?} {} {:?}",
                        left, operator, right
                    ))),
                }
            }
            Expr::Unary { operator, operand } => {
                let operand = self.eval_expr(operand)?;
                match (operator.as_str(), operand.clone()) {
                    ("-", Value::Float(n)) => Ok(Value::Float(-n)),
                    ("!", Value::Boolean(b)) => Ok(Value::Boolean(!b)),
                    _ => Err(RuntimeError::TypeError(format!(
                        "Invalid unary operation: {}{:?}",
                        operator, operand
                    ))),
                }
            }
            Expr::Call { function, arguments } => {
                let function = self.eval_expr(function)?;
                let mut args = Vec::new();
                for arg in arguments {
                    args.push(self.eval_expr(arg)?);
                }
                match function {
                    Value::Function(params, body) => {
                        if params.len() != args.len() {
                            return Err(RuntimeError::TypeError(format!(
                                "Expected {} arguments but got {}",
                                params.len(),
                                args.len()
                            )));
                        }
                        self.environment.push_scope();
                        for (param, arg) in params.iter().zip(args) {
                            self.environment.define(param.clone(), arg);
                        }
                        let result = match self.eval_stmt(&Stmt::Block(body)) {
                            Ok(value) => Ok(value),
                            Err(RuntimeError::Return(value)) => Ok(value),
                            Err(e) => Err(e),
                        };
                        self.environment.pop_scope();
                        result
                    }
                    _ => Err(RuntimeError::TypeError("Not a function".to_string())),
                }
            }
            Expr::Index { array, index } => {
                let array = self.eval_expr(array)?;
                let index = self.eval_expr(index)?;
                match (array, index) {
                    (Value::Array(elements), Value::Float(i)) => {
                        let i = i as usize;
                        if i < elements.len() {
                            Ok(elements[i].clone())
                        } else {
                            Err(RuntimeError::IndexOutOfBounds(i, elements.len()))
                        }
                    }
                    _ => Err(RuntimeError::InvalidArrayAccess),
                }
            }
        }
    }
} 