pub mod core;
pub mod llm;
pub mod medical;
pub mod utils;

use std::collections::HashMap;
use crate::types::Value;

#[derive(Clone)]
pub struct Module {
    pub name: String,
    pub functions: HashMap<String, Value>,
}

impl Module {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            functions: HashMap::new(),
        }
    }

    pub fn register_function(&mut self, name: &str, function: Value) {
        self.functions.insert(name.to_string(), function);
    }
} 