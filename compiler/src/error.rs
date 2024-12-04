use thiserror::Error;
use crate::types::Value;

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

    #[error("Async error: {0}")]
    AsyncError(String),
} 