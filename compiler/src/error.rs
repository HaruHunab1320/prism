use std::error::Error as StdError;
use std::fmt;

#[derive(Debug)]
pub enum RuntimeError {
    DivisionByZero,
    UndefinedVariable(String),
    UndefinedField(String),
    TypeError(String),
    InvalidConfidence(f64),
    InvalidContext(String),
    LLMError(String),
    ParseError(String),
    IndexOutOfBounds(usize, usize),
    InvalidOperation(String),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable '{}'", name),
            RuntimeError::UndefinedField(name) => write!(f, "Undefined field '{}'", name),
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::InvalidConfidence(value) => write!(f, "Invalid confidence value: {}", value),
            RuntimeError::InvalidContext(name) => write!(f, "Invalid context: {}", name),
            RuntimeError::LLMError(msg) => write!(f, "LLM error: {}", msg),
            RuntimeError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            RuntimeError::IndexOutOfBounds(index, len) => write!(f, "Index {} out of bounds (len: {})", index, len),
            RuntimeError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
        }
    }
}

impl StdError for RuntimeError {}

pub trait Error: StdError + Send + Sync + 'static {}

impl<T> Error for T where T: StdError + Send + Sync + 'static {} 