use crate::ast::Stmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    Float(f64),
    String(String),
    Boolean(bool),
    Array(Vec<Value>),
    Function(Vec<String>, Vec<Stmt>),
    Void,
}

impl std::fmt::Display for Value {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Value::Float(n) => write!(f, "{}", n),
            Value::String(s) => write!(f, "\"{}\"", s),
            Value::Boolean(b) => write!(f, "{}", b),
            Value::Array(elements) => {
                write!(f, "[")?;
                for (i, elem) in elements.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", elem)?;
                }
                write!(f, "]")
            }
            Value::Function(params, _) => {
                write!(f, "fn(")?;
                for (i, param) in params.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", param)?;
                }
                write!(f, ")")
            }
            Value::Void => write!(f, "void"),
        }
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Type error: {0}")]
    TypeError(String),
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Index {0} out of bounds for array of length {1}")]
    IndexOutOfBounds(usize, usize),
    #[error("Invalid array access")]
    InvalidArrayAccess,
    #[error("Undefined field: {0}")]
    UndefinedField(String),
    #[error("Return outside of function")]
    Return(Value),
    #[error("Break outside of loop")]
    Break,
    #[error("Continue outside of loop")]
    Continue,
    #[error("Uncaught exception: {0}")]
    Throw(Value),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_value_display() {
        assert_eq!(Value::Float(42.0).to_string(), "42");
        assert_eq!(Value::String("hello".to_string()).to_string(), "\"hello\"");
        assert_eq!(Value::Boolean(true).to_string(), "true");
        assert_eq!(Value::Array(vec![Value::Float(1.0), Value::Float(2.0)]).to_string(), "[1, 2]");
        assert_eq!(Value::Function(vec!["x".to_string(), "y".to_string()], vec![]).to_string(), "fn(x, y)");
        assert_eq!(Value::Void.to_string(), "void");
    }

    #[test]
    fn test_runtime_error_display() {
        assert_eq!(
            RuntimeError::UndefinedVariable("x".to_string()).to_string(),
            "Undefined variable: x"
        );
        assert_eq!(
            RuntimeError::TypeError("invalid operation".to_string()).to_string(),
            "Type error: invalid operation"
        );
        assert_eq!(RuntimeError::DivisionByZero.to_string(), "Division by zero");
        assert_eq!(
            RuntimeError::IndexOutOfBounds(5, 3).to_string(),
            "Index 5 out of bounds for array of length 3"
        );
        assert_eq!(
            RuntimeError::InvalidArrayAccess.to_string(),
            "Invalid array access"
        );
        assert_eq!(
            RuntimeError::UndefinedField("x".to_string()).to_string(),
            "Undefined field: x"
        );
        assert_eq!(
            RuntimeError::Return(Value::Void).to_string(),
            "Return outside of function"
        );
        assert_eq!(
            RuntimeError::Break.to_string(),
            "Break outside of loop"
        );
        assert_eq!(
            RuntimeError::Continue.to_string(),
            "Continue outside of loop"
        );
        assert_eq!(
            RuntimeError::Throw(Value::String("error".to_string())).to_string(),
            "Uncaught exception: \"error\""
        );
    }
} 