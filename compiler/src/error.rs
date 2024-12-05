use std::error::Error;
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
    Other(Box<dyn Error>),
}

impl fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RuntimeError::DivisionByZero => write!(f, "Division by zero"),
            RuntimeError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            RuntimeError::UndefinedField(name) => write!(f, "Undefined field: {}", name),
            RuntimeError::TypeError(msg) => write!(f, "Type error: {}", msg),
            RuntimeError::InvalidConfidence(val) => write!(f, "Invalid confidence value: {}", val),
            RuntimeError::InvalidContext(msg) => write!(f, "Invalid context: {}", msg),
            RuntimeError::LLMError(msg) => write!(f, "LLM error: {}", msg),
            RuntimeError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            RuntimeError::IndexOutOfBounds(idx, len) => write!(f, "Index out of bounds: index {} is out of bounds for array of length {}", idx, len),
            RuntimeError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            RuntimeError::Other(err) => write!(f, "Error: {}", err),
        }
    }
}

impl Error for RuntimeError {}

impl From<Box<dyn Error>> for RuntimeError {
    fn from(error: Box<dyn Error>) -> Self {
        RuntimeError::Other(error)
    }
}

impl From<std::io::Error> for RuntimeError {
    fn from(error: std::io::Error) -> Self {
        RuntimeError::Other(Box::new(error))
    }
}

impl From<String> for RuntimeError {
    fn from(error: String) -> Self {
        RuntimeError::Other(error.into())
    }
}

impl From<&str> for RuntimeError {
    fn from(error: &str) -> Self {
        RuntimeError::Other(error.to_string().into())
    }
} 