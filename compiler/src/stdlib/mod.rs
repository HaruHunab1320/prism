pub mod medical;

use crate::types::Value;
use std::collections::HashMap;

pub struct Stdlib {
    functions: HashMap<String, Value>,
}

impl Stdlib {
    pub fn new() -> Self {
        Self {
            functions: HashMap::new(),
        }
    }

    pub fn register_function(&mut self, name: &str, function: Value) {
        self.functions.insert(name.to_string(), function);
    }

    pub fn get_function(&self, name: &str) -> Option<&Value> {
        self.functions.get(name)
    }
} 