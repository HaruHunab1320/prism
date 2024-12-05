mod ast;
mod context;
mod environment;
mod interpreter;
mod lexer;
mod llm;
mod parser;
mod stdlib;
mod types;
mod value;

pub use ast::{Expr, Stmt};
pub use context::{Context, ContextManager};
pub use environment::Environment;
pub use interpreter::Interpreter;
pub use lexer::{Lexer, Token};
pub use llm::LLMClient;
pub use parser::Parser;
pub use types::Value;

pub mod prelude {
    pub use crate::ast::{Expr, Stmt};
    pub use crate::context::{Context, ContextManager};
    pub use crate::environment::Environment;
    pub use crate::interpreter::Interpreter;
    pub use crate::lexer::{Lexer, Token};
    pub use crate::llm::LLMClient;
    pub use crate::parser::Parser;
    pub use crate::types::Value;
} 