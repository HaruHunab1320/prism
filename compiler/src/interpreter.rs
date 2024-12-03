use std::collections::HashMap;
use crate::ast::{Expr, Stmt, BinaryOperator};
use crate::types::{Context, Confidence};
use crate::context::ContextManager;
use crate::stdlib::StandardLibrary;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Void,
    Float(f64),
    Integer(i64),
    String(String),
    Confidence(Confidence),
    Context(Context),
    Array(Vec<Value>),
}

pub struct Interpreter {
    variables: HashMap<String, Value>,
    context_manager: ContextManager,
    stdlib: StandardLibrary,
}

impl Interpreter {
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
            context_manager: ContextManager::new(0.5), // Default confidence threshold
            stdlib: StandardLibrary::new(),
        }
    }

    pub fn eval(&mut self, stmt: &Stmt) -> Result<Value, String> {
        match stmt {
            Stmt::Expression(expr) => self.eval_expr(expr),
            Stmt::Declaration { name, value } => {
                let evaluated = self.eval_expr(value)?;
                self.variables.insert(name.clone(), evaluated.clone());
                Ok(evaluated)
            },
            Stmt::Assignment { target, value } => {
                let evaluated = self.eval_expr(value)?;
                self.variables.insert(target.clone(), evaluated.clone());
                Ok(evaluated)
            }
        }
    }

    fn eval_expr(&mut self, expr: &Expr) -> Result<Value, String> {
        match expr {
            Expr::Float(f) => Ok(Value::Float(*f)),
            Expr::Integer(i) => Ok(Value::Integer(*i)),
            Expr::String(s) => Ok(Value::String(s.clone())),
            Expr::Identifier(name) => {
                self.variables.get(name)
                    .cloned()
                    .ok_or_else(|| format!("Variable '{}' not found", name))
            },
            
            Expr::ConfidenceValue { value } => {
                let val = self.eval_expr(value)?;
                match val {
                    Value::Float(f) => Ok(Value::Confidence(Confidence::new(f)?)),
                    _ => Err("Confidence value must be a float".to_string()),
                }
            },
            
            Expr::ConfidenceFlow { source, target } => {
                let source_val = self.eval_expr(source)?;
                let target_val = self.eval_expr(target)?;
                match (source_val, target_val) {
                    (Value::Float(s), Value::Float(t)) => Ok(Value::Float(s * t)),
                    _ => Err("Confidence flow requires float values".to_string()),
                }
            },
            
            Expr::ReverseConfidenceFlow { source, target } => {
                let evaluated = self.eval_expr(source)?;
                self.variables.insert(target.to_string(), evaluated.clone());
                Ok(evaluated)
            },
            
            Expr::UncertainIf { condition, high_confidence, medium_confidence, low_confidence } => {
                let cond_val = self.eval_expr(condition)?;
                match cond_val {
                    Value::Float(conf) => {
                        let mut result = Value::Void;
                        if conf >= 0.7 {
                            for stmt in high_confidence {
                                result = self.eval(stmt)?;
                            }
                        } else if conf >= 0.4 {
                            if let Some(stmts) = medium_confidence {
                                for stmt in stmts {
                                    result = self.eval(stmt)?;
                                }
                            }
                        } else if let Some(stmts) = low_confidence {
                            for stmt in stmts {
                                result = self.eval(stmt)?;
                            }
                        }
                        Ok(result)
                    },
                    _ => Err("Condition must evaluate to a confidence value".to_string())
                }
            },
            
            Expr::ContextBlock { context_name, body } => {
                self.context_manager.enter_context(context_name.clone(), 0.5)?;
                let mut result = Value::Void;
                for stmt in body {
                    result = self.eval(stmt)?;
                }
                self.context_manager.exit_context()?;
                Ok(result)
            },
            
            Expr::ContextShift { from, to, body } => {
                let mut results = Vec::new();
                for stmt in body {
                    results.push(self.eval(stmt)?);
                }
                Ok(Value::Context(Context::new(
                    from.clone(),
                    vec![],
                    0.5,
                    vec![to.clone()]
                )?))
            },
            
            Expr::BinaryOp { op, left, right } => {
                let left_val = self.eval_expr(left)?;
                let right_val = self.eval_expr(right)?;
                match op {
                    BinaryOperator::Add => {
                        match (&left_val, &right_val) {
                            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a + b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a + b)),
                            _ => Err("Add operation requires matching numeric types".to_string())
                        }
                    },
                    BinaryOperator::Subtract => {
                        match (&left_val, &right_val) {
                            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a - b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a - b)),
                            _ => Err("Subtract operation requires matching numeric types".to_string())
                        }
                    },
                    BinaryOperator::Multiply => {
                        match (&left_val, &right_val) {
                            (Value::Integer(a), Value::Integer(b)) => Ok(Value::Integer(a * b)),
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a * b)),
                            _ => Err("Multiply operation requires matching numeric types".to_string())
                        }
                    },
                    BinaryOperator::Divide => {
                        match (&left_val, &right_val) {
                            (Value::Integer(a), Value::Integer(b)) => {
                                if *b == 0 {
                                    Err("Division by zero".to_string())
                                } else {
                                    Ok(Value::Integer(a / b))
                                }
                            },
                            (Value::Float(a), Value::Float(b)) => {
                                if *b == 0.0 {
                                    Err("Division by zero".to_string())
                                } else {
                                    Ok(Value::Float(a / b))
                                }
                            },
                            _ => Err("Divide operation requires matching numeric types".to_string())
                        }
                    },
                    BinaryOperator::ConfidenceAnd => {
                        match (left_val, right_val) {
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.min(b))),
                            _ => Err("Confidence AND operation requires float values".to_string())
                        }
                    },
                    BinaryOperator::ConfidenceOr => {
                        match (left_val, right_val) {
                            (Value::Float(a), Value::Float(b)) => Ok(Value::Float(a.max(b))),
                            _ => Err("Confidence OR operation requires float values".to_string())
                        }
                    },
                }
            },
            
            Expr::Verify { sources: _, threshold: _, body } => {
                // Placeholder for verification logic
                let mut result = Value::Void;
                for stmt in body {
                    result = self.eval(stmt)?;
                }
                Ok(result)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::{Expr, Stmt};

    #[test]
    fn test_confidence_declaration() {
        let mut interpreter = Interpreter::new();
        
        let stmt = Stmt::Declaration {
            name: "x".to_string(),
            value: Expr::Float(0.8),
        };
        
        let result = interpreter.eval(&stmt).unwrap();
        assert_eq!(result, Value::Float(0.8));
    }

    #[test]
    fn test_confidence_flow() {
        let mut interpreter = Interpreter::new();
        
        // First declare x = 0.8
        let decl = Stmt::Declaration {
            name: "x".to_string(),
            value: Expr::Float(0.8),
        };
        interpreter.eval(&decl).unwrap();
        
        // Then do x ~> 0.7
        let flow = Stmt::Expression(Expr::ConfidenceFlow {
            source: Box::new(Expr::Identifier("x".to_string())),
            target: Box::new(Expr::Float(0.7)),
        });
        
        let result = interpreter.eval(&flow).unwrap();
        match result {
            Value::Float(f) => {
                let expected = 0.8 * 0.7;
                let diff = (f - expected).abs();
                assert!(diff < 1e-10, "Expected approximately {}, got {}", expected, f);
            },
            _ => panic!("Expected float value"),
        }
    }
} 