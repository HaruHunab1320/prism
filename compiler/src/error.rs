use thiserror::Error;
use google_generative_ai_rs::v1::errors::GoogleAPIError;

#[derive(Error, Debug)]
pub enum RuntimeError {
    #[error("Type error: {0}")]
    TypeError(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Undefined variable: {0}")]
    UndefinedVariable(String),

    #[error("Break statement outside loop")]
    Break,

    #[error("Continue statement outside loop")]
    Continue,

    #[error("User error: {0}")]
    UserError(String),

    #[error("Division by zero")]
    DivisionByZero,

    #[error("Index {0} out of bounds for array of length {1}")]
    IndexOutOfBounds(usize, usize),

    #[error("Undefined field: {0}")]
    UndefinedField(String),

    #[error("Async error: {0}")]
    AsyncError(String),

    #[error("API error: {0}")]
    APIError(String),
}

impl From<GoogleAPIError> for RuntimeError {
    fn from(error: GoogleAPIError) -> Self {
        RuntimeError::APIError(error.to_string())
    }
} 