mod ast;
mod error;
mod interpreter;
mod lexer;
mod parser;
mod stdlib;
mod value;
mod llm;

pub use interpreter::Interpreter;
pub use value::Value;
pub use lexer::Lexer;
pub use parser::Parser;

pub trait Error: std::error::Error + Send + Sync + 'static {}

impl<T> Error for T where T: std::error::Error + Send + Sync + 'static {}