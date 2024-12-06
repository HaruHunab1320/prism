use std::fmt;
use std::error::Error as StdError;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Self {
            message: message.to_string(),
        }
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {}

// Implement From<Box<dyn StdError + Send + Sync>> for Error
impl From<Box<dyn StdError + Send + Sync>> for Error {
    fn from(error: Box<dyn StdError + Send + Sync>) -> Self {
        Error::new(&error.to_string())
    }
}

// Implement Send and Sync since our error type needs to be Send + Sync
unsafe impl Send for Error {}
unsafe impl Sync for Error {}
