use std::path::PathBuf;
use dotenv::dotenv;

pub fn init() {
    // Try to load .env from workspace root first
    let root_env = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".env");
    
    if root_env.exists() {
        dotenv::from_path(root_env).ok();
    } else {
        // Fallback to default dotenv behavior
        dotenv().ok();
    }
}

pub mod token;
pub mod lexer;
pub mod parser;
pub mod ast;
pub mod interpreter;
pub mod environment;
pub mod value;
pub mod error;
pub mod module;
pub mod types;
pub mod confidence;
pub mod context;
pub mod llm;
pub mod stdlib;

pub use interpreter::Interpreter;
