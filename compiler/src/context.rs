use std::collections::HashMap;
use std::fmt;
use serde::{Serialize, Deserialize};
use crate::types::Value;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Context {
    name: String,
    confidence: f64,
    values: HashMap<String, Value>,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Context({}, confidence: {})", self.name, self.confidence)
    }
}

impl Context {
    pub fn new(name: String) -> Self {
        Self {
            name,
            confidence: 1.0,
            values: HashMap::new(),
        }
    }

    pub fn get_confidence(&self) -> f64 {
        self.confidence
    }

    pub fn set_confidence(&mut self, confidence: f64) {
        self.confidence = confidence;
    }

    pub fn get_value(&self, name: &str) -> Option<&Value> {
        self.values.get(name)
    }

    pub fn set_value(&mut self, name: String, value: Value) {
        self.values.insert(name, value);
    }

    pub fn remove_value(&mut self, name: &str) -> Option<Value> {
        self.values.remove(name)
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }
} 