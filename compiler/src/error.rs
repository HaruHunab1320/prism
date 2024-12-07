use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub enum PrismError {
    Parse(String),
    Runtime(String),
    ModuleNotFound(String),
    CircularDependency(String),
    SymbolNotFound { module: String, symbol: String },
    UndefinedVariable(String),
    TypeError(String),
    InvalidArgument(String),
    InvalidOperation(String),
    ImportError(String),
    IOError(std::io::Error),
}

impl fmt::Display for PrismError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PrismError::Parse(msg) => write!(f, "Parse error: {}", msg),
            PrismError::Runtime(msg) => write!(f, "Runtime error: {}", msg),
            PrismError::ModuleNotFound(name) => write!(f, "Module not found: {}", name),
            PrismError::CircularDependency(name) => write!(f, "Circular dependency detected: {}", name),
            PrismError::SymbolNotFound { module, symbol } => {
                write!(f, "Symbol '{}' not found in module '{}'", symbol, module)
            }
            PrismError::UndefinedVariable(name) => write!(f, "Undefined variable: {}", name),
            PrismError::TypeError(msg) => write!(f, "Type error: {}", msg),
            PrismError::InvalidArgument(msg) => write!(f, "Invalid argument: {}", msg),
            PrismError::InvalidOperation(msg) => write!(f, "Invalid operation: {}", msg),
            PrismError::ImportError(msg) => write!(f, "Import error: {}", msg),
            PrismError::IOError(err) => write!(f, "IO error: {}", err),
        }
    }
}

impl Error for PrismError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            PrismError::IOError(err) => Some(err),
            _ => None,
        }
    }
}

impl From<std::io::Error> for PrismError {
    fn from(err: std::io::Error) -> Self {
        PrismError::IOError(err)
    }
}

pub type Result<T> = std::result::Result<T, Box<dyn Error + Send + Sync>>;
