use std::io;
use thiserror::Error;

pub type Result<T> = std::result::Result<T, PrismError>;

#[derive(Error, Debug)]
pub enum PrismError {
    #[error("Runtime error: {0}")]
    RuntimeError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Module not found: {0}")]
    ModuleNotFound(String),

    #[error("Module already exists: {0}")]
    ModuleAlreadyExists(String),

    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    #[error("JSON error: {0}")]
    JsonError(#[from] serde_json::Error),
}
