use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use crate::value::Value;
use crate::interpreter::Interpreter;

pub mod core;
pub mod llm;
pub mod medical;
pub mod utils;

#[derive(Debug, Clone)]
pub struct Module {
    name: String,
    functions: HashMap<String, Value>,
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

pub fn register_core_functions(interpreter: &mut Interpreter) {
    core::register_core_functions(interpreter);
}

pub fn register_utils_functions(interpreter: &mut Interpreter) {
    utils::register_utils_functions(interpreter);
} 