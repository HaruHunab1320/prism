use thiserror::Error;

#[derive(Error, Debug)]
pub enum PrismError {
    #[error("Module not found: {0}")]
    ModuleNotFound(String),
    #[error("Module already exists: {0}")]
    ModuleAlreadyExists(String),
    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),
    #[error("Invalid operator: {0:?}")]
    InvalidOperator(crate::token::Token),
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),
    #[error("Invalid operation: {0}")]
    InvalidOperation(String),
    #[error("Division by zero")]
    DivisionByZero,
    #[error("Import error: {0}")]
    ImportError(String),
    #[error("Parse error: {0}")]
    ParseError(String),
    #[error("Lexer error: {0}")]
    LexerError(String),
    #[error("Runtime error: {0}")]
    RuntimeError(String),
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("Serialization error: {0}")]
    SerializationError(#[from] serde_json::Error),
    #[error("HTTP error: {0}")]
    HttpError(String),
}

impl From<reqwest::Error> for PrismError {
    fn from(err: reqwest::Error) -> Self {
        PrismError::HttpError(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, PrismError>;
