use std::io;
use serde_json;

pub type Result<T> = std::result::Result<T, PrismError>;

#[derive(Debug)]
pub enum PrismError {
    IO(io::Error),
    ParseError(String),
    TypeError(String),
    RuntimeError(String),
    Serialization(serde_json::Error),
    ModuleNotFound(String),
    ModuleAlreadyExists(String),
    UndefinedVariable(String),
    InvalidOperation(String),
    InvalidArgument(String),
}

impl From<io::Error> for PrismError {
    fn from(err: io::Error) -> Self {
        PrismError::IO(err)
    }
}

impl From<serde_json::Error> for PrismError {
    fn from(err: serde_json::Error) -> Self {
        PrismError::Serialization(err)
    }
}

impl std::fmt::Display for PrismError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PrismError::IO(err) => write!(f, "IO error: {}", err),
            PrismError::ParseError(msg) => write!(f, "Parse error: {}", msg),
            PrismError::TypeError(msg) => write!(f, "Type error: {}", msg),
            PrismError::RuntimeError(msg) => write!(f, "Runtime error: {}", msg),
            PrismError::Serialization(err) => write!(f, "Serialization error: {}", err),
            PrismError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            PrismError::ModuleAlreadyExists(name) => write!(f, "Module already exists: {}", name),
            PrismError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            PrismError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            PrismError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
        }
    }
}

impl std::error::Error for PrismError {}
