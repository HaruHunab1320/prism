use std::collections::HashMap;
use crate::types::Value;

#[derive(Clone)]
pub struct Environment {
    values: HashMap<String, Value>,
    confidence_values: HashMap<String, f64>,
    current_context: Option<String>,
}

impl Environment {
    pub fn new() -> Self {
        Self {
            values: HashMap::new(),
            confidence_values: HashMap::new(),
            current_context: None,
        }
    }

    pub fn define(&mut self, name: &str, value: Value) {
        self.values.insert(name.to_string(), value);
    }

    pub fn define_with_confidence(&mut self, name: &str, value: Value, confidence: f64) {
        self.values.insert(name.to_string(), value);
        self.confidence_values.insert(name.to_string(), confidence);
    }

    pub fn get(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn get_confidence(&self, name: &str) -> Option<f64> {
        self.confidence_values.get(name).copied()
    }

    pub fn assign(&mut self, name: &str, value: Value) -> bool {
        if self.values.contains_key(name) {
            self.values.insert(name.to_string(), value);
            true
        } else {
            false
        }
    }

    pub fn set_context(&mut self, context: Option<String>) {
        self.current_context = context;
    }

    pub fn get_current_context(&self) -> Option<&String> {
        self.current_context.as_ref()
    }
} 