use std::fmt;
use std::error::Error as StdError;

pub type BoxError = Box<dyn StdError + Send + Sync>;

#[derive(Debug)]
pub struct Error {
    message: String,
}

impl Error {
    pub fn new(message: &str) -> Self {
        Error {
            message: message.to_string(),
        }
    }

    pub fn boxed(message: &str) -> BoxError {
        Box::new(Self::new(message))
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl StdError for Error {}

unsafe impl Send for Error {}
unsafe impl Sync for Error {}
