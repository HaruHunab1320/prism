use std::collections::HashMap;
use crate::types::Value;

#[derive(Clone)]
pub struct ContextManager {
    threshold: f64,
    contexts: HashMap<String, Context>,
}

#[derive(Clone)]
pub struct Context {
    name: String,
    confidence: f64,
    values: HashMap<String, Value>,
}

impl ContextManager {
    pub fn new(threshold: f64) -> Self {
        Self {
            threshold,
            contexts: HashMap::new(),
        }
    }

    pub fn get_threshold(&self) -> f64 {
        self.threshold
    }

    pub fn set_threshold(&mut self, threshold: f64) {
        self.threshold = threshold;
    }

    pub fn create_context(&mut self, name: String, confidence: f64) -> &mut Context {
        self.contexts.entry(name.clone()).or_insert_with(|| Context {
            name,
            confidence,
            values: HashMap::new(),
        })
    }

    pub fn get_context(&self, name: &str) -> Option<&Context> {
        self.contexts.get(name)
    }

    pub fn get_context_mut(&mut self, name: &str) -> Option<&mut Context> {
        self.contexts.get_mut(name)
    }

    pub fn remove_context(&mut self, name: &str) -> Option<Context> {
        self.contexts.remove(name)
    }
}

impl Context {
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